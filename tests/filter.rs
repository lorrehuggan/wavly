use std::time::Duration;

#[test]
fn filter_keeps_pending_rows_visible() {
    let filter = wavly::filter::parse_filter_query("120-128 Am").expect("filter");
    let entry = wavly::sort::TrackEntry {
        filename: "track.mp3".into(),
        status_label: "pending".into(),
        status_rank: 0,
        bpm: None,
        key: None,
        standard_key: None,
        length: None,
        format: "MP3".into(),
    };

    assert!(filter.matches_entry(&entry));
}

#[test]
fn filter_matches_done_tracks_by_bpm_and_key() {
    let filter = wavly::filter::parse_filter_query("120-128 8A").expect("filter");
    let matching = wavly::sort::TrackEntry {
        filename: "track.mp3".into(),
        status_label: "done".into(),
        status_rank: 2,
        bpm: Some(124.0),
        key: Some("A minor (8A)".into()),
        standard_key: Some("A minor".into()),
        length: Some(Duration::from_secs(180)),
        format: "MP3".into(),
    };
    let non_matching = wavly::sort::TrackEntry {
        filename: "track2.mp3".into(),
        status_label: "done".into(),
        status_rank: 2,
        bpm: Some(140.0),
        key: Some("C major (8B)".into()),
        standard_key: Some("C major".into()),
        length: Some(Duration::from_secs(180)),
        format: "MP3".into(),
    };

    assert!(filter.matches_entry(&matching));
    assert!(!filter.matches_entry(&non_matching));
}
