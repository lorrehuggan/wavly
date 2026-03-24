use anyhow::Result;
use clap::Parser;
use rayon::prelude::*;
use std::{path::PathBuf, process::ExitCode, sync::mpsc, thread};

use wavly::{analysis, cli, scanner, tui};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err:#}");
            ExitCode::FAILURE
        }
    }
}

fn install_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        ratatui::restore();
        original_hook(panic_info);
    }));
}

fn run() -> Result<()> {
    let cli = cli::Cli::parse();
    let files = scanner::discover_audio_files(&cli.paths, !cli.no_recursive)?;

    if files.is_empty() {
        return Err(anyhow::anyhow!("no audio files found"));
    }

    let (tx, rx) = mpsc::channel();
    let worker_files: Vec<(usize, PathBuf)> = files.iter().cloned().enumerate().collect();
    let worker = thread::spawn(move || {
        worker_files
            .into_par_iter()
            .for_each_with(tx, |tx, (index, path)| {
                let _ = tx.send(tui::WorkerMessage::Started(index));
                let result = analysis::analyze_file(&path).map_err(|err| err.to_string());
                let _ = tx.send(tui::WorkerMessage::Finished { index, result });
            });
    });

    install_panic_hook();
    let mut terminal = ratatui::try_init()?;
    let mut app = tui::App::new(files);
    let result = tui::run(&mut terminal, &mut app, &rx);
    ratatui::restore();
    let _ = worker.join();
    result?;

    Ok(())
}
