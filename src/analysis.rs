use crate::audio::decode_audio_file;
use anyhow::{Context, Result};
use std::{fmt::Write as _, path::Path, time::Duration};

#[derive(Debug, Clone)]
pub struct TrackAnalysis {
    pub bpm: f64,
    pub key_name: String,
    pub key_numerical: String,
    pub duration: Duration,
}

pub fn analyze_file(path: &Path) -> Result<TrackAnalysis> {
    let decoded = decode_audio_file(path)?;
    let result = stratum_dsp::analyze_audio(
        &decoded.samples,
        decoded.sample_rate,
        stratum_dsp::AnalysisConfig::default(),
    )
    .with_context(|| format!("failed to analyze {}", path.display()))?;

    Ok(TrackAnalysis {
        bpm: result.bpm as f64,
        key_name: result.key.name().to_string(),
        key_numerical: result.key.numerical().to_string(),
        duration: decoded.duration,
    })
}

pub fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;

    let mut formatted = String::new();
    let _ = write!(&mut formatted, "{minutes:02}:{seconds:02}");
    formatted
}
