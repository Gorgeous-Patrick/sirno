//! Terminal UI for common entry defaults.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState};
use ratatui::{DefaultTerminal, Frame};

use crate::surface::cli::tui::run_terminal_ui;
use crate::surface::context::resolve_lake_directory;
use crate::surface::error::CommandError;
use crate::{
    CheckMode, Entry, EntryDirectory, EntryDirectoryCheckSettings, EntryId, EntryMetadata,
    StructuralSettings,
};

const CATEGORY_FIELD: &str = "category";
const BELONGS_FIELD: &str = "belongs";
const PREREQUISITE_FIELD: &str = "prerequisite";

// sirno:witness:interfaces:begin
const DEFAULT_ENTRIES: [DefaultEntrySpec; 8] = [
    DefaultEntrySpec {
        id: "category",
        name: "Category",
        desc: "An entry that other entries can be categorized by.",
        category: &["category"],
        belongs: &[],
        prerequisite: &[],
        body: "Categorize an entry by this entry to use it as a category target.\n",
    },
    DefaultEntrySpec {
        id: "meta",
        name: "Meta",
        desc: "An entry that defines project principles, vocabulary, and documentation method.",
        category: &["category"],
        belongs: &[],
        prerequisite: &[],
        body: "Defines how this project should be understood and developed.\n",
    },
    DefaultEntrySpec {
        id: "concept",
        name: "Concept",
        desc: "A named idea that compresses project knowledge.",
        category: &["category"],
        belongs: &[],
        prerequisite: &[],
        body: "A concept gives a stable name to compressed project knowledge.\n",
    },
    DefaultEntrySpec {
        id: "narrative",
        name: "Narrative",
        desc: "A route through concepts for a reader.",
        category: &["category"],
        belongs: &[],
        prerequisite: &[],
        body: "A narrative records an order in which a reader can understand concepts.\n",
    },
    DefaultEntrySpec {
        id: "structural-field",
        name: "Structural Field",
        desc: "A metadata field that carries operational Sirno structure.",
        category: &["concept"],
        belongs: &[],
        prerequisite: &[],
        body: "A structural field is a configured metadata field that Sirno reads as project structure.\n",
    },
    DefaultEntrySpec {
        id: "belongs",
        name: "Belongs",
        desc: "A structural field that places an entry in a review neighborhood.",
        category: &["concept"],
        belongs: &["structural-field"],
        prerequisite: &["structural-field"],
        body: "`belongs` places an entry in a named review neighborhood.\n",
    },
    DefaultEntrySpec {
        id: "refines",
        name: "Refines",
        desc: "A structural field from a specific entry to the broader entries it makes concrete.",
        category: &["concept"],
        belongs: &["structural-field"],
        prerequisite: &["structural-field"],
        body: "`refines` records a refinement edge from a specific entry to a broader entry.\n",
    },
    DefaultEntrySpec {
        id: "prerequisite",
        name: "Prerequisite",
        desc: "A structural field that defines a knowledge dependency between entries.",
        category: &["concept"],
        belongs: &["structural-field"],
        prerequisite: &["structural-field"],
        body: "`prerequisite` records knowledge an entry expects the reader to understand first.\n",
    },
];
// sirno:witness:interfaces:end

/// Run the interactive entry-default maintenance UI.
pub(crate) fn run(config_path: &Path, lake_path: Option<&Path>) -> Result<ExitCode, CommandError> {
    run_terminal_ui(|terminal| run_app(terminal, config_path, lake_path))
}

fn run_app(
    terminal: &mut DefaultTerminal, config_path: &Path, lake_path: Option<&Path>,
) -> Result<ExitCode, CommandError> {
    let mut app =
        EntryDefaultsTui::load(config_path.to_path_buf(), lake_path.map(Path::to_path_buf))?;
    loop {
        terminal.draw(|frame| app.render(frame)).map_err(CommandError::TerminalUi)?;
        if let Event::Key(key) = event::read().map_err(CommandError::TerminalUi)? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            // sirno:witness:interfaces:begin
            match key.code {
                | KeyCode::Char('q') | KeyCode::Esc => return Ok(ExitCode::SUCCESS),
                | KeyCode::Char('j') | KeyCode::Down => app.next(),
                | KeyCode::Char('k') | KeyCode::Up => app.previous(),
                | KeyCode::Char('i') => app.insert_selected()?,
                | KeyCode::Char('a') => app.insert_all_missing()?,
                | _ => {}
            }
            // sirno:witness:interfaces:end
        }
    }
}

#[derive(Debug)]
struct EntryDefaultsTui {
    config_path: PathBuf,
    lake_override: Option<PathBuf>,
    lake_path: PathBuf,
    settings: EntryDirectoryCheckSettings,
    rows: Vec<EntryDefaultRow>,
    selected: usize,
    message: String,
}

impl EntryDefaultsTui {
    fn load(config_path: PathBuf, lake_override: Option<PathBuf>) -> Result<Self, CommandError> {
        let mut app = Self {
            config_path,
            lake_override,
            lake_path: PathBuf::new(),
            settings: EntryDirectoryCheckSettings::default(),
            rows: Vec::new(),
            selected: 0,
            message: String::new(),
        };
        app.reload(
            "j/k or arrows move; i inserts selected; a inserts all missing; q quits".to_owned(),
        )?;
        Ok(app)
    }

    fn reload(&mut self, action: String) -> Result<(), CommandError> {
        let selected_id = self.rows.get(self.selected).map(|row| row.spec.id);
        let (lake_path, settings) =
            resolve_lake_directory(self.lake_override.as_deref(), &self.config_path)?;
        let report =
            EntryDirectory::new(&lake_path).check_with_settings(CheckMode::Review, &settings)?;
        self.rows = default_entry_rows(report.entries(), &settings.structural);
        self.selected = selected_id
            .and_then(|id| self.rows.iter().position(|row| row.spec.id == id))
            .unwrap_or(0);
        self.lake_path = lake_path;
        self.settings = settings;
        self.message = format!("{action}; {}", review_check_summary(&report));
        Ok(())
    }

    fn next(&mut self) {
        self.selected = (self.selected + 1).min(self.rows.len().saturating_sub(1));
    }

    fn previous(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    fn insert_selected(&mut self) -> Result<(), CommandError> {
        let row = self.rows[self.selected].clone();
        match row.status {
            | EntryDefaultStatus::Missing => {
                self.create_default(row.spec)?;
                self.reload(format!("inserted {}", row.spec.id))
            }
            | EntryDefaultStatus::Present => {
                self.message = format!("{} is already present", row.spec.id);
                Ok(())
            }
            | EntryDefaultStatus::NeedsCategoryMarker => {
                self.message = format!("{} is present but needs category: category", row.spec.id);
                Ok(())
            }
        }
    }

    fn insert_all_missing(&mut self) -> Result<(), CommandError> {
        let missing = self
            .rows
            .iter()
            .filter(|row| row.status == EntryDefaultStatus::Missing)
            .map(|row| row.spec)
            .collect::<Vec<_>>();
        if missing.is_empty() {
            self.message = "all default entries are already present".to_owned();
            return Ok(());
        }
        for spec in &missing {
            self.create_default(spec)?;
        }
        self.reload(format!("inserted {} default entries", missing.len()))
    }

    fn create_default(&self, spec: &DefaultEntrySpec) -> Result<(), CommandError> {
        let entry = spec.entry(&self.settings.structural)?;
        EntryDirectory::new(&self.lake_path).create_entry(&entry)?;
        Ok(())
    }

    fn render(&self, frame: &mut Frame<'_>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(5), Constraint::Length(3)])
            .split(frame.area());

        let header = Row::new(["Entry", "Status", "Default Fields", "Description"])
            .style(Style::default().add_modifier(Modifier::BOLD));
        let rows = self.rows.iter().map(|row| {
            Row::new([
                Cell::from(row.spec.id),
                Cell::from(row.status.label()),
                Cell::from(row.default_fields.as_str()),
                Cell::from(row.spec.desc),
            ])
        });
        let table = Table::new(
            rows,
            [
                Constraint::Length(22),
                Constraint::Length(18),
                Constraint::Length(30),
                Constraint::Min(24),
            ],
        )
        .header(header)
        .block(Block::default().title("Entry Defaults").borders(Borders::ALL))
        .row_highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");
        let mut state = TableState::default().with_selected(Some(self.selected));
        frame.render_stateful_widget(table, chunks[0], &mut state);

        let footer = Paragraph::new(self.message.as_str())
            .block(Block::default().title("Keys").borders(Borders::ALL));
        frame.render_widget(footer, chunks[1]);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct EntryDefaultRow {
    spec: &'static DefaultEntrySpec,
    status: EntryDefaultStatus,
    default_fields: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EntryDefaultStatus {
    Missing,
    Present,
    NeedsCategoryMarker,
}

impl EntryDefaultStatus {
    fn label(self) -> &'static str {
        match self {
            | Self::Missing => "missing",
            | Self::Present => "present",
            | Self::NeedsCategoryMarker => "needs category",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DefaultEntrySpec {
    id: &'static str,
    name: &'static str,
    desc: &'static str,
    category: &'static [&'static str],
    belongs: &'static [&'static str],
    prerequisite: &'static [&'static str],
    body: &'static str,
}

impl DefaultEntrySpec {
    fn entry(self, structural: &StructuralSettings) -> Result<Entry, CommandError> {
        let mut metadata = EntryMetadata::new(self.name, self.desc)?;
        self.set_targets(&mut metadata, structural, CATEGORY_FIELD, self.category);
        self.set_targets(&mut metadata, structural, BELONGS_FIELD, self.belongs);
        self.set_targets(&mut metadata, structural, PREREQUISITE_FIELD, self.prerequisite);
        Ok(Entry::new(entry_id(self.id), metadata, self.body))
    }

    fn set_targets(
        self, metadata: &mut EntryMetadata, structural: &StructuralSettings, field: &str,
        targets: &[&str],
    ) {
        if targets.is_empty() || !structural.contains_field(field) {
            return;
        }
        metadata.set_structural_targets(field, targets.iter().map(|target| entry_id(target)));
    }
}

fn default_entry_rows(entries: &[Entry], structural: &StructuralSettings) -> Vec<EntryDefaultRow> {
    let entries_by_id =
        entries.iter().map(|entry| (entry.id.clone(), entry)).collect::<BTreeMap<_, _>>();
    let category_targets = entries
        .iter()
        .flat_map(|entry| entry.metadata.structural_targets_for(CATEGORY_FIELD))
        .cloned()
        .collect::<BTreeSet<_>>();
    DEFAULT_ENTRIES
        .iter()
        .map(|spec| {
            let id = entry_id(spec.id);
            let status = match entries_by_id.get(&id) {
                | None => EntryDefaultStatus::Missing,
                | Some(entry) if category_targets.contains(&id) && !has_category_marker(entry) => {
                    EntryDefaultStatus::NeedsCategoryMarker
                }
                | Some(_) => EntryDefaultStatus::Present,
            };
            EntryDefaultRow { spec, status, default_fields: spec.configured_fields(structural) }
        })
        .collect()
}

impl DefaultEntrySpec {
    fn configured_fields(self, structural: &StructuralSettings) -> String {
        let mut fields = Vec::new();
        self.push_field_summary(&mut fields, structural, CATEGORY_FIELD, self.category);
        self.push_field_summary(&mut fields, structural, BELONGS_FIELD, self.belongs);
        self.push_field_summary(&mut fields, structural, PREREQUISITE_FIELD, self.prerequisite);
        if fields.is_empty() { "-".to_owned() } else { fields.join("; ") }
    }

    fn push_field_summary(
        self, fields: &mut Vec<String>, structural: &StructuralSettings, field: &str,
        targets: &[&str],
    ) {
        if targets.is_empty() || !structural.contains_field(field) {
            return;
        }
        fields.push(format!("{field}={}", targets.join(",")));
    }
}

fn has_category_marker(entry: &Entry) -> bool {
    let category_id = entry_id(CATEGORY_FIELD);
    entry.metadata.structural_targets_for(CATEGORY_FIELD).iter().any(|id| id == &category_id)
}

fn entry_id(raw: &str) -> EntryId {
    EntryId::new(raw).unwrap_or_else(|error| panic!("invalid built-in entry id `{raw}`: {error}"))
}

fn review_check_summary(report: &crate::EntryDirectoryReport) -> String {
    let diagnostics =
        report.file_diagnostics().len() + report.structural_report().diagnostics().len();
    if report.has_errors() {
        format!("review check: errors ({diagnostics} diagnostics)")
    } else if report.is_clean() {
        "review check: ok".to_owned()
    } else {
        format!("review check: warnings ({diagnostics} diagnostics)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StructuralFieldSettings;

    fn spec(id: &str) -> DefaultEntrySpec {
        *DEFAULT_ENTRIES.iter().find(|spec| spec.id == id).unwrap()
    }

    #[test]
    fn default_entry_uses_only_configured_structural_fields() {
        let empty = spec("belongs").entry(&StructuralSettings::default()).unwrap();
        assert_eq!(empty.metadata.structural_fields().count(), 0);

        let structural = StructuralSettings::from_fields([
            (CATEGORY_FIELD, StructuralFieldSettings::default()),
            (BELONGS_FIELD, StructuralFieldSettings::default()),
            (PREREQUISITE_FIELD, StructuralFieldSettings::default()),
        ]);
        let full = spec("belongs").entry(&structural).unwrap();

        assert_eq!(
            full.metadata
                .structural_targets_for(CATEGORY_FIELD)
                .iter()
                .map(EntryId::as_str)
                .collect::<Vec<_>>(),
            ["concept"]
        );
        assert_eq!(
            full.metadata
                .structural_targets_for(BELONGS_FIELD)
                .iter()
                .map(EntryId::as_str)
                .collect::<Vec<_>>(),
            ["structural-field"]
        );
    }

    #[test]
    fn rows_report_missing_and_category_marker_status() {
        let mut concept_metadata = EntryMetadata::new("Concept", "A concept.").unwrap();
        concept_metadata.push_structural_target(CATEGORY_FIELD, entry_id("meta"));
        let concept = Entry::new(entry_id("concept"), concept_metadata, "");
        let meta = Entry::new(entry_id("meta"), EntryMetadata::new("Meta", "Meta.").unwrap(), "");

        let rows = default_entry_rows(&[concept, meta], &StructuralSettings::default());

        let meta = rows.iter().find(|row| row.spec.id == "meta").unwrap();
        let category = rows.iter().find(|row| row.spec.id == "category").unwrap();
        assert_eq!(meta.status, EntryDefaultStatus::NeedsCategoryMarker);
        assert_eq!(category.status, EntryDefaultStatus::Missing);
    }
}
