use std::time::Duration;

#[test]
fn sorts_by_bpm_and_toggles_direction() {
    let mut rows = vec![
        wavly::sort::TrackEntry {
            filename: "b.mp3".into(),
            status_label: "done".into(),
            status_rank: 2,
            bpm: Some(128.0),
            key: Some("Am (8A)".into()),
            standard_key: Some("A minor".into()),
            length: Some(Duration::from_secs(180)),
            format: "MP3".into(),
        },
        wavly::sort::TrackEntry {
            filename: "a.wav".into(),
            status_label: "done".into(),
            status_rank: 2,
            bpm: Some(120.0),
            key: Some("C (8B)".into()),
            standard_key: Some("C major".into()),
            length: Some(Duration::from_secs(200)),
            format: "WAV".into(),
        },
        wavly::sort::TrackEntry {
            filename: "c.flac".into(),
            status_label: "pending".into(),
            status_rank: 0,
            bpm: None,
            key: None,
            standard_key: None,
            length: None,
            format: "FLAC".into(),
        },
    ];

    let state = wavly::sort::SortState::new(wavly::sort::SortColumn::Bpm);
    wavly::sort::sort_entries(&mut rows, state);

    assert_eq!(rows[0].filename, "a.wav");
    assert_eq!(rows[1].filename, "b.mp3");
    assert_eq!(rows[2].filename, "c.flac");

    let mut state = state;
    state.toggle(wavly::sort::SortColumn::Bpm);
    wavly::sort::sort_entries(&mut rows, state);

    assert_eq!(rows[0].filename, "b.mp3");
    assert_eq!(rows[1].filename, "a.wav");
    assert_eq!(rows[2].filename, "c.flac");
}

#[test]
fn sorts_by_filename_case_insensitively() {
    let mut rows = vec![
        wavly::sort::TrackEntry {
            filename: "Zulu.mp3".into(),
            status_label: "done".into(),
            status_rank: 2,
            bpm: Some(128.0),
            key: Some("Am (8A)".into()),
            standard_key: Some("A minor".into()),
            length: Some(Duration::from_secs(180)),
            format: "MP3".into(),
        },
        wavly::sort::TrackEntry {
            filename: "alpha.wav".into(),
            status_label: "done".into(),
            status_rank: 2,
            bpm: Some(120.0),
            key: Some("C (8B)".into()),
            standard_key: Some("C major".into()),
            length: Some(Duration::from_secs(200)),
            format: "WAV".into(),
        },
    ];

    wavly::sort::sort_entries(
        &mut rows,
        wavly::sort::SortState::new(wavly::sort::SortColumn::Filename),
    );

    assert_eq!(rows[0].filename, "alpha.wav");
    assert_eq!(rows[1].filename, "Zulu.mp3");
}
