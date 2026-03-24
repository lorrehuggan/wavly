use std::{f32::consts::PI, fs::File, io::BufWriter, path::Path};

use tempfile::tempdir;

#[test]
fn analyzes_a_test_wav_end_to_end() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("test-track.wav");
    write_test_wav(&path);

    let analysis =
        wavly::analysis::analyze_file(Path::new(&path)).expect("analysis should succeed");

    assert!(
        (analysis.bpm - 120.0).abs() <= 2.5,
        "unexpected bpm: {}",
        analysis.bpm
    );
    assert!(
        !analysis.key_name.is_empty(),
        "key name should not be empty"
    );
    assert!(
        !analysis.key_numerical.is_empty(),
        "key numerical should not be empty"
    );
    assert!(
        (analysis.duration.as_secs_f64() - 8.0).abs() < 0.05,
        "unexpected duration: {:?}",
        analysis.duration
    );
}

fn write_test_wav(path: &Path) {
    let sample_rate = 44_100;
    let bpm = 120.0;
    let duration_secs = 8.0;
    let total_samples = (sample_rate as f32 * duration_secs) as usize;
    let beat_interval = sample_rate as f32 * 60.0 / bpm;
    let click_len = (sample_rate as f32 * 0.03) as usize;
    let chord_gain = 0.14;
    let click_gain = 0.85;

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let file = File::create(path).expect("create wav file");
    let writer = BufWriter::new(file);
    let mut wav = hound::WavWriter::new(writer, spec).expect("wav writer");

    for i in 0..total_samples {
        let t = i as f32 / sample_rate as f32;

        let chord = chord_gain
            * ((2.0 * PI * 220.0 * t).sin()
                + (2.0 * PI * 261.63 * t).sin()
                + (2.0 * PI * 329.63 * t).sin())
            / 3.0;

        let beat_phase = (i as f32) % beat_interval;
        let click = if beat_phase < click_len as f32 {
            let env = 1.0 - beat_phase / click_len as f32;
            click_gain * env
        } else {
            0.0
        };

        let sample = (chord + click).clamp(-1.0, 1.0);
        wav.write_sample((sample * i16::MAX as f32) as i16)
            .expect("write sample");
    }

    wav.finalize().expect("finalize wav");
}
