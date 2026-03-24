#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyFormat {
    Standard,
    Camelot,
    OpenKey,
}

impl KeyFormat {
    pub fn next(self) -> Self {
        match self {
            Self::Standard => Self::Camelot,
            Self::Camelot => Self::OpenKey,
            Self::OpenKey => Self::Standard,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Standard => "Standard",
            Self::Camelot => "Camelot",
            Self::OpenKey => "Open Key",
        }
    }
}

pub fn format_key(standard_name: &str, format: KeyFormat) -> Option<String> {
    let (note, mode) = parse_any_key(standard_name)?;

    match format {
        KeyFormat::Standard => Some(standard_shorthand(note, mode)),
        KeyFormat::Camelot => Some(camelot_code(note, mode).to_string()),
        KeyFormat::OpenKey => Some(open_key_code(note, mode).to_string()),
    }
}

fn standard_shorthand(note: &str, mode: Mode) -> String {
    match mode {
        Mode::Major => note.to_string(),
        Mode::Minor => format!("{note}m"),
    }
}

pub fn normalize_key_label(input: &str) -> Option<String> {
    let (note, mode) = parse_any_key(input)?;
    Some(match mode {
        Mode::Major => format!("{note} major"),
        Mode::Minor => format!("{note} minor"),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Major,
    Minor,
}

fn parse_any_key(input: &str) -> Option<(&'static str, Mode)> {
    let input = input.trim();
    let lower = input.to_ascii_lowercase();

    if let Some(note) = lower.strip_suffix(" major") {
        return Some((canonical_note(note)?, Mode::Major));
    }

    if let Some(note) = lower.strip_suffix(" minor") {
        return Some((canonical_note(note)?, Mode::Minor));
    }

    if let Some(note) = lower.strip_suffix('m') {
        return Some((canonical_note(note)?, Mode::Minor));
    }

    if let Some(code) = parse_camelot(&lower) {
        return Some(code);
    }

    if let Some(code) = parse_open_key(&lower) {
        return Some(code);
    }

    Some((canonical_note(&lower)?, Mode::Major))
}

fn canonical_note(note: &str) -> Option<&'static str> {
    match note.trim() {
        "c" => Some("C"),
        "c#" | "db" => Some("Db"),
        "d" => Some("D"),
        "d#" | "eb" => Some("Eb"),
        "e" => Some("E"),
        "f" => Some("F"),
        "f#" | "gb" => Some("F#"),
        "g" => Some("G"),
        "g#" | "ab" => Some("Ab"),
        "a" => Some("A"),
        "a#" | "bb" => Some("Bb"),
        "b" => Some("B"),
        _ => None,
    }
}

fn camelot_code(note: &str, mode: Mode) -> &'static str {
    match (note, mode) {
        ("C", Mode::Major) => "8B",
        ("Db", Mode::Major) => "3B",
        ("D", Mode::Major) => "10B",
        ("Eb", Mode::Major) => "5B",
        ("E", Mode::Major) => "12B",
        ("F", Mode::Major) => "7B",
        ("F#", Mode::Major) => "2B",
        ("G", Mode::Major) => "9B",
        ("Ab", Mode::Major) => "4B",
        ("A", Mode::Major) => "11B",
        ("Bb", Mode::Major) => "6B",
        ("B", Mode::Major) => "1B",
        ("A", Mode::Minor) => "8A",
        ("Bb", Mode::Minor) => "3A",
        ("B", Mode::Minor) => "10A",
        ("C", Mode::Minor) => "5A",
        ("Db", Mode::Minor) => "12A",
        ("D", Mode::Minor) => "7A",
        ("Eb", Mode::Minor) => "2A",
        ("E", Mode::Minor) => "9A",
        ("F", Mode::Minor) => "4A",
        ("F#", Mode::Minor) => "11A",
        ("G", Mode::Minor) => "6A",
        ("Ab", Mode::Minor) => "1A",
        _ => "8B",
    }
}

fn open_key_code(note: &str, mode: Mode) -> &'static str {
    match (note, mode) {
        ("C", Mode::Major) => "1d",
        ("G", Mode::Major) => "2d",
        ("D", Mode::Major) => "3d",
        ("A", Mode::Major) => "4d",
        ("E", Mode::Major) => "5d",
        ("B", Mode::Major) => "6d",
        ("F#", Mode::Major) => "7d",
        ("Db", Mode::Major) => "8d",
        ("Ab", Mode::Major) => "9d",
        ("Eb", Mode::Major) => "10d",
        ("Bb", Mode::Major) => "11d",
        ("F", Mode::Major) => "12d",
        ("A", Mode::Minor) => "1m",
        ("E", Mode::Minor) => "2m",
        ("B", Mode::Minor) => "3m",
        ("F#", Mode::Minor) => "4m",
        ("Db", Mode::Minor) => "5m",
        ("Ab", Mode::Minor) => "6m",
        ("Eb", Mode::Minor) => "7m",
        ("Bb", Mode::Minor) => "8m",
        ("F", Mode::Minor) => "9m",
        ("C", Mode::Minor) => "10m",
        ("G", Mode::Minor) => "11m",
        ("D", Mode::Minor) => "12m",
        _ => "1d",
    }
}

fn parse_camelot(input: &str) -> Option<(&'static str, Mode)> {
    let (number, mode_char) = input.trim().split_at(input.len().saturating_sub(1));
    let hour = number.parse::<u8>().ok()?;
    let mode = match mode_char {
        "a" => Mode::Minor,
        "b" => Mode::Major,
        _ => return None,
    };

    match (hour, mode) {
        (1, Mode::Minor) => Some(("Ab", Mode::Minor)),
        (1, Mode::Major) => Some(("B", Mode::Major)),
        (2, Mode::Minor) => Some(("Eb", Mode::Minor)),
        (2, Mode::Major) => Some(("F#", Mode::Major)),
        (3, Mode::Minor) => Some(("Bb", Mode::Minor)),
        (3, Mode::Major) => Some(("Db", Mode::Major)),
        (4, Mode::Minor) => Some(("F", Mode::Minor)),
        (4, Mode::Major) => Some(("Ab", Mode::Major)),
        (5, Mode::Minor) => Some(("C", Mode::Minor)),
        (5, Mode::Major) => Some(("Eb", Mode::Major)),
        (6, Mode::Minor) => Some(("G", Mode::Minor)),
        (6, Mode::Major) => Some(("Bb", Mode::Major)),
        (7, Mode::Minor) => Some(("D", Mode::Minor)),
        (7, Mode::Major) => Some(("F", Mode::Major)),
        (8, Mode::Minor) => Some(("A", Mode::Minor)),
        (8, Mode::Major) => Some(("C", Mode::Major)),
        (9, Mode::Minor) => Some(("E", Mode::Minor)),
        (9, Mode::Major) => Some(("G", Mode::Major)),
        (10, Mode::Minor) => Some(("B", Mode::Minor)),
        (10, Mode::Major) => Some(("D", Mode::Major)),
        (11, Mode::Minor) => Some(("F#", Mode::Minor)),
        (11, Mode::Major) => Some(("A", Mode::Major)),
        (12, Mode::Minor) => Some(("Db", Mode::Minor)),
        (12, Mode::Major) => Some(("E", Mode::Major)),
        _ => None,
    }
}

fn parse_open_key(input: &str) -> Option<(&'static str, Mode)> {
    let (number, mode_char) = input.trim().split_at(input.len().saturating_sub(1));
    let hour = number.parse::<u8>().ok()?;
    let mode = match mode_char {
        "m" => Mode::Minor,
        "d" => Mode::Major,
        _ => return None,
    };

    match (hour, mode) {
        (1, Mode::Major) => Some(("C", Mode::Major)),
        (2, Mode::Major) => Some(("G", Mode::Major)),
        (3, Mode::Major) => Some(("D", Mode::Major)),
        (4, Mode::Major) => Some(("A", Mode::Major)),
        (5, Mode::Major) => Some(("E", Mode::Major)),
        (6, Mode::Major) => Some(("B", Mode::Major)),
        (7, Mode::Major) => Some(("F#", Mode::Major)),
        (8, Mode::Major) => Some(("Db", Mode::Major)),
        (9, Mode::Major) => Some(("Ab", Mode::Major)),
        (10, Mode::Major) => Some(("Eb", Mode::Major)),
        (11, Mode::Major) => Some(("Bb", Mode::Major)),
        (12, Mode::Major) => Some(("F", Mode::Major)),
        (1, Mode::Minor) => Some(("A", Mode::Minor)),
        (2, Mode::Minor) => Some(("E", Mode::Minor)),
        (3, Mode::Minor) => Some(("B", Mode::Minor)),
        (4, Mode::Minor) => Some(("F#", Mode::Minor)),
        (5, Mode::Minor) => Some(("Db", Mode::Minor)),
        (6, Mode::Minor) => Some(("Ab", Mode::Minor)),
        (7, Mode::Minor) => Some(("Eb", Mode::Minor)),
        (8, Mode::Minor) => Some(("Bb", Mode::Minor)),
        (9, Mode::Minor) => Some(("F", Mode::Minor)),
        (10, Mode::Minor) => Some(("C", Mode::Minor)),
        (11, Mode::Minor) => Some(("G", Mode::Minor)),
        (12, Mode::Minor) => Some(("D", Mode::Minor)),
        _ => None,
    }
}
