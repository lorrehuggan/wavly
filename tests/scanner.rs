use std::fs::File;

use tempfile::tempdir;

#[test]
fn discovers_audio_files_recursively() {
    let dir = tempdir().expect("tempdir");
    let root_audio = dir.path().join("track-a.mp3");
    let root_text = dir.path().join("notes.txt");
    let nested_dir = dir.path().join("nested");
    let nested_audio = nested_dir.join("track-b.wav");
    let deeper_dir = nested_dir.join("deeper");
    let deeper_audio = deeper_dir.join("track-c.flac");

    File::create(&root_audio).expect("root audio");
    File::create(&root_text).expect("root text");
    std::fs::create_dir_all(&deeper_dir).expect("nested dirs");
    File::create(&nested_audio).expect("nested audio");
    File::create(&deeper_audio).expect("deeper audio");

    let files = wavly::scanner::discover_audio_files(&[dir.path().to_path_buf()], true)
        .expect("discover files");

    let expected = vec![deeper_audio, nested_audio, root_audio];

    assert_eq!(
        files, expected,
        "recursive scan should include nested audio files"
    );
}

#[test]
fn discovers_audio_files_non_recursively() {
    let dir = tempdir().expect("tempdir");
    let root_audio = dir.path().join("track-a.mp3");
    let nested_dir = dir.path().join("nested");
    let nested_audio = nested_dir.join("track-b.wav");

    File::create(&root_audio).expect("root audio");
    std::fs::create_dir_all(&nested_dir).expect("nested dir");
    File::create(&nested_audio).expect("nested audio");

    let files = wavly::scanner::discover_audio_files(&[dir.path().to_path_buf()], false)
        .expect("discover files");

    assert_eq!(
        files,
        vec![root_audio],
        "non-recursive scan should stay flat"
    );
}
