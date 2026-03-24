use crate::{key_format::normalize_key_label, sort::TrackEntry};

#[derive(Debug, Clone, PartialEq)]
pub struct TrackFilter {
    pub bpm_min: Option<f64>,
    pub bpm_max: Option<f64>,
    pub key: Option<String>,
}

impl TrackFilter {
    pub fn matches_entry(&self, entry: &TrackEntry) -> bool {
        if entry.status_rank != 2 {
            return true;
        }

        if let Some(min) = self.bpm_min
            && entry.bpm.is_none_or(|bpm| bpm < min)
        {
            return false;
        }

        if let Some(max) = self.bpm_max
            && entry.bpm.is_none_or(|bpm| bpm > max)
        {
            return false;
        }

        if let Some(expected_key) = self.key.as_deref()
            && entry.standard_key.as_deref() != Some(expected_key)
        {
            return false;
        }

        true
    }

    pub fn describe(&self) -> String {
        let mut parts = Vec::new();

        if self.bpm_min.is_some() || self.bpm_max.is_some() {
            parts.push(match (self.bpm_min, self.bpm_max) {
                (Some(min), Some(max)) => format!("BPM {min:.0}-{max:.0}"),
                (Some(min), None) => format!("BPM ≥ {min:.0}"),
                (None, Some(max)) => format!("BPM ≤ {max:.0}"),
                (None, None) => String::new(),
            });
        }

        if let Some(key) = &self.key {
            parts.push(format!("Key {key}"));
        }

        if parts.is_empty() {
            "All tracks".to_string()
        } else {
            parts.join("  |  ")
        }
    }
}

pub fn parse_filter_query(input: &str) -> Option<TrackFilter> {
    let mut filter = TrackFilter {
        bpm_min: None,
        bpm_max: None,
        key: None,
    };

    let mut found = false;

    for raw_token in input.split(|c: char| c.is_whitespace() || c == ',') {
        let token = raw_token.trim();
        if token.is_empty() {
            continue;
        }

        let token = token
            .strip_prefix("bpm=")
            .or_else(|| token.strip_prefix("bpm:"))
            .unwrap_or(token);
        let token = token
            .strip_prefix("key=")
            .or_else(|| token.strip_prefix("key:"))
            .unwrap_or(token);

        if let Some((min, max)) = parse_bpm_range(token) {
            filter.bpm_min = Some(filter.bpm_min.map_or(min, |current| current.max(min)));
            filter.bpm_max = Some(filter.bpm_max.map_or(max, |current| current.min(max)));
            found = true;
            continue;
        }

        if let Some(key) = normalize_key_label(token) {
            filter.key = Some(key);
            found = true;
            continue;
        }
    }

    if found { Some(filter) } else { None }
}

fn parse_bpm_range(token: &str) -> Option<(f64, f64)> {
    let (left, right) = token.split_once('-')?;
    let min = left.trim().parse::<f64>().ok()?;
    let max = right.trim().parse::<f64>().ok()?;
    Some((min.min(max), min.max(max)))
}

#[cfg(test)]
mod tests {
    use super::parse_filter_query;

    #[test]
    fn parses_bpm_and_key_query() {
        let filter = parse_filter_query("120-128 Am").expect("filter");
        assert_eq!(filter.bpm_min, Some(120.0));
        assert_eq!(filter.bpm_max, Some(128.0));
        assert_eq!(filter.key.as_deref(), Some("A minor"));
    }

    #[test]
    fn parses_camelot_and_open_key() {
        let camelot = parse_filter_query("8A").expect("camelot");
        assert_eq!(camelot.key.as_deref(), Some("A minor"));

        let open_key = parse_filter_query("1d").expect("open key");
        assert_eq!(open_key.key.as_deref(), Some("C major"));
    }
}
