use crate::{
    analysis::{format_duration, TrackAnalysis},
    filter::{parse_filter_query, TrackFilter},
    key_format::{format_key, KeyFormat},
    sort::{self, SortColumn, SortDirection, SortState, TrackEntry},
};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Cell, HighlightSpacing, Paragraph, Row, Table, TableState},
    DefaultTerminal,
};
use std::{path::PathBuf, sync::mpsc::Receiver, time::Duration};

#[derive(Debug, Clone)]
pub enum RowStatus {
    Pending,
    Analyzing,
    Done,
    Error(String),
}

impl RowStatus {
    fn label(&self) -> String {
        match self {
            Self::Pending => "pending".to_string(),
            Self::Analyzing => "analyzing".to_string(),
            Self::Done => "done".to_string(),
            Self::Error(message) => format!("error: {message}"),
        }
    }

    fn rank(&self) -> u8 {
        match self {
            Self::Pending => 0,
            Self::Analyzing => 1,
            Self::Done => 2,
            Self::Error(_) => 3,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TrackRow {
    path: PathBuf,
    status: RowStatus,
    analysis: Option<TrackAnalysis>,
}

impl TrackRow {
    fn new(path: PathBuf) -> Self {
        Self {
            path,
            status: RowStatus::Pending,
            analysis: None,
        }
    }

    fn format_label(&self) -> String {
        self.path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_uppercase())
            .unwrap_or_else(|| "UNKNOWN".to_string())
    }

    fn to_entry(&self, key_format: KeyFormat) -> TrackEntry {
        let (bpm, key, length) = match &self.analysis {
            Some(analysis) => (
                Some(analysis.bpm),
                format_key(&analysis.key_name, key_format).or_else(|| {
                    Some(format!(
                        "{} ({})",
                        analysis.key_name, analysis.key_numerical
                    ))
                }),
                Some(analysis.duration),
            ),
            None => (None, None, None),
        };

        TrackEntry {
            filename: self.path.display().to_string(),
            status_label: self.status.label(),
            status_rank: self.status.rank(),
            bpm,
            key,
            standard_key: self
                .analysis
                .as_ref()
                .map(|analysis| analysis.key_name.clone()),
            length,
            format: self.format_label(),
        }
    }
}

pub enum WorkerMessage {
    Started(usize),
    Finished {
        index: usize,
        result: std::result::Result<TrackAnalysis, String>,
    },
}

pub struct App {
    rows: Vec<TrackRow>,
    selected: Option<usize>,
    table_state: TableState,
    should_quit: bool,
    completed: usize,
    total: usize,
    sort_state: SortState,
    key_format: KeyFormat,
    filter_mode: bool,
    filter_input: String,
    active_filter: Option<TrackFilter>,
    visible_count: usize,
}

impl App {
    pub fn new(paths: Vec<PathBuf>) -> Self {
        let total = paths.len();
        let mut table_state = TableState::default();
        if total > 0 {
            table_state.select_first();
        }

        Self {
            rows: paths.into_iter().map(TrackRow::new).collect(),
            selected: if total > 0 { Some(0) } else { None },
            table_state,
            should_quit: false,
            completed: 0,
            total,
            sort_state: SortState::new(SortColumn::Filename),
            key_format: KeyFormat::Standard,
            filter_mode: false,
            filter_input: String::new(),
            active_filter: None,
            visible_count: total,
        }
    }

    pub fn apply_message(&mut self, message: WorkerMessage) {
        match message {
            WorkerMessage::Started(index) => {
                if let Some(row) = self.rows.get_mut(index) {
                    row.status = RowStatus::Analyzing;
                }
            }
            WorkerMessage::Finished { index, result } => {
                if let Some(row) = self.rows.get_mut(index) {
                    match result {
                        Ok(analysis) => {
                            row.analysis = Some(analysis);
                            row.status = RowStatus::Done;
                        }
                        Err(err) => {
                            row.status = RowStatus::Error(err);
                        }
                    }
                }
                self.completed = self.completed.saturating_add(1);
            }
        }
    }

    fn status_line(&self, entries: &[TrackEntry]) -> String {
        let errors = self
            .rows
            .iter()
            .filter(|row| matches!(row.status, RowStatus::Error(_)))
            .count();

        let selection = self
            .selected
            .and_then(|i| entries.get(i))
            .map(|entry| entry.filename.clone())
            .unwrap_or_else(|| "no selection".to_string());

        format!(
            "Analyzed {}/{} files  |  {} errors  |  Key: {}  |  Filter: {}  |  Selected: {}",
            self.completed,
            self.total,
            errors,
            self.key_format.label(),
            self.active_filter
                .as_ref()
                .map(|filter| filter.describe())
                .unwrap_or_else(|| "All tracks".to_string()),
            selection
        )
    }

    fn sort_by_column(&mut self, column: SortColumn) {
        self.sort_state.toggle(column);
    }

    fn entries_for_render(&self) -> Vec<TrackEntry> {
        let mut entries = self
            .rows
            .iter()
            .map(|row| row.to_entry(self.key_format))
            .filter(|entry| {
                self.active_filter
                    .as_ref()
                    .is_none_or(|filter| filter.matches_entry(entry))
            })
            .collect::<Vec<_>>();
        sort::sort_entries(&mut entries, self.sort_state);
        entries
    }

    fn clamp_selection(&mut self, len: usize) {
        match self.selected {
            Some(_) if len == 0 => self.selected = None,
            Some(index) if index >= len => self.selected = Some(len - 1),
            None if len > 0 => self.selected = Some(0),
            _ => {}
        }
    }

    fn move_next(&mut self) {
        let max = self.visible_count.saturating_sub(1);
        self.selected = Some(match self.selected {
            Some(index) => index.saturating_add(1).min(max),
            None => 0,
        });
    }

    fn move_previous(&mut self) {
        self.selected = Some(match self.selected {
            Some(0) | None => 0,
            Some(index) => index.saturating_sub(1),
        });
    }

    fn move_first(&mut self) {
        if self.visible_count > 0 {
            self.selected = Some(0);
        }
    }

    fn move_last(&mut self) {
        self.selected = if self.visible_count == 0 {
            None
        } else {
            Some(self.visible_count - 1)
        };
    }

    fn page_up(&mut self) {
        self.selected = Some(match self.selected {
            Some(index) => index.saturating_sub(10),
            None => 0,
        });
    }

    fn page_down(&mut self) {
        let max = self.visible_count.saturating_sub(1);
        self.selected = Some(match self.selected {
            Some(index) => index.saturating_add(10).min(max),
            None => 0,
        });
    }

    fn cycle_key_format(&mut self) {
        self.key_format = self.key_format.next();
    }

    fn open_filter(&mut self) {
        self.filter_mode = true;
        self.filter_input.clear();
    }

    fn clear_filter(&mut self) {
        self.active_filter = None;
        self.filter_input.clear();
        self.filter_mode = false;
    }

    fn apply_filter(&mut self) {
        if self.filter_input.trim().is_empty() {
            self.active_filter = None;
        } else if let Some(filter) = parse_filter_query(&self.filter_input) {
            self.active_filter = Some(filter);
        }
        self.filter_mode = false;
    }

    fn cancel_filter(&mut self) {
        self.filter_mode = false;
        self.filter_input.clear();
    }

    fn handle_filter_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => self.cancel_filter(),
            KeyCode::Enter => self.apply_filter(),
            KeyCode::Backspace => {
                self.filter_input.pop();
            }
            KeyCode::Char(c) => {
                self.filter_input.push(c);
            }
            _ => {}
        }
    }

    fn on_key(&mut self, key: KeyCode) {
        if self.filter_mode {
            self.handle_filter_key(key);
            return;
        }

        match key {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('j') | KeyCode::Down => self.move_next(),
            KeyCode::Char('k') | KeyCode::Up => self.move_previous(),
            KeyCode::Char('g') | KeyCode::Home => self.move_first(),
            KeyCode::Char('G') | KeyCode::End => self.move_last(),
            KeyCode::PageUp => self.page_up(),
            KeyCode::PageDown => self.page_down(),
            KeyCode::Char('1') => self.sort_by_column(SortColumn::Filename),
            KeyCode::Char('2') => self.sort_by_column(SortColumn::Status),
            KeyCode::Char('3') => self.sort_by_column(SortColumn::Bpm),
            KeyCode::Char('4') => self.sort_by_column(SortColumn::Key),
            KeyCode::Char('5') => self.sort_by_column(SortColumn::Length),
            KeyCode::Char('6') => self.sort_by_column(SortColumn::Format),
            KeyCode::Char('t') => self.cycle_key_format(),
            KeyCode::Char('/') => self.open_filter(),
            KeyCode::Char('c') => self.clear_filter(),
            _ => {}
        }
    }
}

pub fn run(
    terminal: &mut DefaultTerminal,
    app: &mut App,
    rx: &Receiver<WorkerMessage>,
) -> Result<()> {
    loop {
        while let Ok(message) = rx.try_recv() {
            app.apply_message(message);
        }

        terminal.draw(|frame| render(frame, app))?;

        if app.should_quit {
            break;
        }

        if event::poll(Duration::from_millis(100))? {
            let event = event::read()?;
            if let Event::Key(key) = event {
                app.on_key(key.code);
            }
        }
    }

    Ok(())
}

fn render(frame: &mut ratatui::Frame, app: &mut App) {
    let entries = app.entries_for_render();
    let visible = entries.len();
    app.visible_count = visible;
    app.clamp_selection(visible);
    app.table_state.select(app.selected);

    let areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(3),
            Constraint::Min(6),
            Constraint::Length(5),
        ])
        .split(frame.area());

    let title = Paragraph::new(Line::from(vec![
        Span::from("Wavly").bold().cyan(),
        Span::raw("  music analysis for crate-diggers and set prep").dim(),
    ]))
    .block(
        Block::bordered()
            .title("Library")
            .title_style(Style::default().cyan()),
    );
    frame.render_widget(title, areas[0]);

    let summary = summary_line(app, visible);
    let summary = Paragraph::new(summary).block(
        Block::bordered()
            .title("Status")
            .title_style(Style::default().cyan()),
    );
    frame.render_widget(summary, areas[1]);

    let rows = entries
        .iter()
        .map(|entry| {
            Row::new(vec![
                Cell::from(entry.filename.clone()),
                Cell::from(entry.status_label.clone()),
                Cell::from(
                    entry
                        .bpm
                        .map(|bpm| format!("{:.2}", bpm))
                        .unwrap_or_else(|| "-".to_string()),
                ),
                Cell::from(entry.key.clone().unwrap_or_else(|| "-".to_string())),
                Cell::from(
                    entry
                        .length
                        .map(format_duration)
                        .unwrap_or_else(|| "-".to_string()),
                ),
                Cell::from(entry.format.clone()),
            ])
            .style(status_style(entry.status_rank))
        })
        .collect::<Vec<_>>();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(40),
            Constraint::Length(12),
            Constraint::Length(10),
            Constraint::Percentage(22),
            Constraint::Length(8),
            Constraint::Length(10),
        ],
    )
    .header(sort_header(app.sort_state))
    .row_highlight_style(
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol("▶ ")
    .highlight_spacing(HighlightSpacing::Always)
    .block(
        Block::bordered()
            .title("Track Analysis")
            .title_style(Style::default().cyan())
            .borders(Borders::ALL),
    );

    frame.render_stateful_widget(table, areas[2], &mut app.table_state);

    let footer = if app.filter_mode {
        let filter_text = if app.filter_input.is_empty() {
            "type BPM range or key".dim().to_string()
        } else {
            app.filter_input.clone()
        };

        Paragraph::new(Text::from(vec![
            Line::from(vec![
                Span::from("/").bold().cyan(),
                Span::raw(" filter ").dim(),
                Span::from("Enter").bold().cyan(),
                Span::raw(" apply  ").dim(),
                Span::from("Esc").bold().cyan(),
                Span::raw(" cancel").dim(),
            ]),
            Line::from(vec![Span::raw(filter_text)]),
        ]))
        .block(
            Block::bordered()
                .title("Filter")
                .title_style(Style::default().cyan()),
        )
    } else {
        Paragraph::new(Text::from(vec![
            Line::from(vec![
                Span::from("q").bold().cyan(),
                Span::raw(" quit  ").dim(),
                Span::from("/").bold().cyan(),
                Span::raw(" filter  ").dim(),
                Span::from("c").bold().cyan(),
                Span::raw(" clear  ").dim(),
                Span::from("j/k").bold().cyan(),
                Span::raw(" or arrows  ").dim(),
                Span::from("1-6").bold().cyan(),
                Span::raw(" sort  ").dim(),
                Span::from("t").bold().cyan(),
                Span::raw(" key format").dim(),
            ]),
            Line::from(vec![Span::raw(app.status_line(&entries)).dim()]),
        ]))
        .block(
            Block::bordered()
                .title("Keys")
                .title_style(Style::default().cyan()),
        )
    };
    frame.render_widget(footer, areas[3]);
}

fn summary_line(app: &App, visible: usize) -> Line<'static> {
    let mut spans = vec![
        Span::from("files ").dim(),
        Span::from(format!("{}/{}", app.completed, app.total))
            .bold()
            .green(),
        Span::raw("  "),
        Span::from("visible ").dim(),
        Span::from(visible.to_string()).bold().cyan(),
        Span::raw("  "),
        Span::from("errors ").dim(),
        Span::from(
            app.rows
                .iter()
                .filter(|row| matches!(row.status, RowStatus::Error(_)))
                .count()
                .to_string(),
        )
        .bold()
        .red(),
        Span::raw("  "),
        Span::from("sort ").dim(),
        Span::from(match app.sort_state.direction {
            SortDirection::Asc => "asc",
            SortDirection::Desc => "desc",
        })
        .bold()
        .magenta(),
        Span::raw("  "),
        Span::from("key ").dim(),
        Span::from(app.key_format.label()).bold().yellow(),
    ];

    if let Some(filter) = &app.active_filter {
        spans.extend([
            Span::raw("  "),
            Span::from("filter ").dim(),
            Span::from(filter.describe()).bold().cyan(),
        ]);
    }

    Line::from(spans)
}

fn sort_header(state: SortState) -> Row<'static> {
    let labels = [
        (SortColumn::Filename, "Filename"),
        (SortColumn::Status, "Status"),
        (SortColumn::Bpm, "BPM"),
        (SortColumn::Key, "Key"),
        (SortColumn::Length, "Length"),
        (SortColumn::Format, "Format"),
    ];

    Row::new(
        labels
            .into_iter()
            .map(|(column, label)| {
                if column == state.column {
                    let arrow = match state.direction {
                        SortDirection::Asc => "↑",
                        SortDirection::Desc => "↓",
                    };
                    format!("{label} {arrow}")
                } else {
                    label.to_string()
                }
            })
            .collect::<Vec<_>>(),
    )
    .style(Style::default().bold().fg(Color::Cyan))
}

fn status_style(rank: u8) -> Style {
    match rank {
        0 => Style::default().fg(Color::DarkGray),
        1 => Style::default().fg(Color::Yellow),
        2 => Style::default().fg(Color::Green),
        _ => Style::default().fg(Color::Red),
    }
}

#[cfg(test)]
mod tests {
    use super::App;
    use std::path::PathBuf;

    #[test]
    fn navigation_initializes_with_first_row_selected() {
        let app = App::new(vec![PathBuf::from("a.mp3"), PathBuf::from("b.wav")]);
        assert_eq!(app.selected, Some(0));
    }

    #[test]
    fn navigation_moves_between_rows() {
        let mut app = App::new(vec![PathBuf::from("a.mp3"), PathBuf::from("b.wav")]);
        app.move_next();
        assert_eq!(app.selected, Some(1));
        app.move_previous();
        assert_eq!(app.selected, Some(0));
        app.move_last();
        assert_eq!(app.selected, Some(1));
        app.move_first();
        assert_eq!(app.selected, Some(0));
    }
}
