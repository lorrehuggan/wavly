use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "wavly",
    about = "Analyze audio files for BPM, key, and duration"
)]
pub struct Cli {
    #[arg(required = true)]
    pub paths: Vec<PathBuf>,

    #[arg(long = "no-recursive", default_value_t = false)]
    pub no_recursive: bool,
}
