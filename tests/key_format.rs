#[test]
fn converts_standard_to_camelot() {
    assert_eq!(
        wavly::key_format::format_key("C major", wavly::key_format::KeyFormat::Camelot),
        Some("8B".to_string())
    );
    assert_eq!(
        wavly::key_format::format_key("A minor", wavly::key_format::KeyFormat::Camelot),
        Some("8A".to_string())
    );
}

#[test]
fn converts_standard_to_open_key() {
    assert_eq!(
        wavly::key_format::format_key("C major", wavly::key_format::KeyFormat::OpenKey),
        Some("1d".to_string())
    );
    assert_eq!(
        wavly::key_format::format_key("A minor", wavly::key_format::KeyFormat::OpenKey),
        Some("1m".to_string())
    );
}

#[test]
fn cycles_formats_in_order() {
    use wavly::key_format::KeyFormat;

    assert_eq!(KeyFormat::Standard.next(), KeyFormat::Camelot);
    assert_eq!(KeyFormat::Camelot.next(), KeyFormat::OpenKey);
    assert_eq!(KeyFormat::OpenKey.next(), KeyFormat::Standard);
}
