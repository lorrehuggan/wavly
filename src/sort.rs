use std::{cmp::Ordering, time::Duration};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    Filename,
    Status,
    Bpm,
    Key,
    Length,
    Format,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SortState {
    pub column: SortColumn,
    pub direction: SortDirection,
}

impl SortState {
    pub fn new(column: SortColumn) -> Self {
        Self {
            column,
            direction: SortDirection::Asc,
        }
    }

    pub fn toggle(&mut self, column: SortColumn) {
        if self.column == column {
            self.direction = match self.direction {
                SortDirection::Asc => SortDirection::Desc,
                SortDirection::Desc => SortDirection::Asc,
            };
        } else {
            self.column = column;
            self.direction = SortDirection::Asc;
        }
    }
}

#[derive(Debug, Clone)]
pub struct TrackEntry {
    pub filename: String,
    pub status_label: String,
    pub status_rank: u8,
    pub bpm: Option<f64>,
    pub key: Option<String>,
    pub standard_key: Option<String>,
    pub length: Option<Duration>,
    pub format: String,
}

pub fn sort_entries(entries: &mut [TrackEntry], state: SortState) {
    entries.sort_by(|a, b| {
        let primary = compare_entries(a, b, state.column, state.direction);

        if primary == Ordering::Equal {
            a.filename.cmp(&b.filename)
        } else {
            primary
        }
    });
}

fn compare_entries(
    a: &TrackEntry,
    b: &TrackEntry,
    column: SortColumn,
    direction: SortDirection,
) -> Ordering {
    match column {
        SortColumn::Filename => cmp_text(&a.filename, &b.filename, direction),
        SortColumn::Status => match direction {
            SortDirection::Asc => a
                .status_rank
                .cmp(&b.status_rank)
                .then_with(|| a.status_label.cmp(&b.status_label)),
            SortDirection::Desc => b
                .status_rank
                .cmp(&a.status_rank)
                .then_with(|| b.status_label.cmp(&a.status_label)),
        },
        SortColumn::Bpm => compare_option_f64(a.bpm, b.bpm, direction),
        SortColumn::Key => compare_option_string(a.key.as_deref(), b.key.as_deref(), direction),
        SortColumn::Length => compare_option_duration(a.length, b.length, direction),
        SortColumn::Format => cmp_text(&a.format, &b.format, direction),
    }
}

fn cmp_text(a: &str, b: &str, direction: SortDirection) -> Ordering {
    match direction {
        SortDirection::Asc => a.to_lowercase().cmp(&b.to_lowercase()),
        SortDirection::Desc => b.to_lowercase().cmp(&a.to_lowercase()),
    }
}

fn compare_option_f64(a: Option<f64>, b: Option<f64>, direction: SortDirection) -> Ordering {
    match (a, b) {
        (Some(a), Some(b)) => match direction {
            SortDirection::Asc => a.partial_cmp(&b).unwrap_or(Ordering::Equal),
            SortDirection::Desc => b.partial_cmp(&a).unwrap_or(Ordering::Equal),
        },
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn compare_option_string(a: Option<&str>, b: Option<&str>, direction: SortDirection) -> Ordering {
    match (a, b) {
        (Some(a), Some(b)) => match direction {
            SortDirection::Asc => a.to_lowercase().cmp(&b.to_lowercase()),
            SortDirection::Desc => b.to_lowercase().cmp(&a.to_lowercase()),
        },
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn compare_option_duration(
    a: Option<Duration>,
    b: Option<Duration>,
    direction: SortDirection,
) -> Ordering {
    match (a, b) {
        (Some(a), Some(b)) => match direction {
            SortDirection::Asc => a.cmp(&b),
            SortDirection::Desc => b.cmp(&a),
        },
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}
