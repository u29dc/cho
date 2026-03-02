//! Command palette data model.

use crate::routes::Workspace;

/// Palette command sections.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteSection {
    /// Route-specific actions.
    Context,
    /// Route navigation actions.
    Navigate,
    /// Global app actions.
    Global,
}

impl PaletteSection {
    /// Display label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Context => "Context",
            Self::Navigate => "Navigate",
            Self::Global => "Global",
        }
    }
}

/// Palette command behavior.
#[derive(Debug, Clone)]
pub enum PaletteActionKind {
    /// Navigate to route id.
    Navigate(String),
    /// Refresh current route.
    Refresh,
    /// Exit app.
    Quit,
    /// Toggle tree visibility.
    ToggleTree,
    /// Prompt for bank account filter.
    PromptBankAccount,
    /// Prompt for self-assessment user.
    PromptSelfAssessmentUser,
    /// Prompt for payroll year.
    PromptPayrollYear,
    /// Prompt for payroll period.
    PromptPayrollPeriod,
    /// Prompt for id/URL required by get-only route.
    PromptTargetId(String),
}

/// One command palette entry.
#[derive(Debug, Clone)]
pub struct PaletteAction {
    /// Primary label.
    pub title: String,
    /// Optional right-side context label.
    pub context: String,
    /// Section bucket.
    pub section: PaletteSection,
    /// Action behavior.
    pub kind: PaletteActionKind,
    /// Search keywords.
    pub keywords: Vec<String>,
}

/// Palette runtime state.
#[derive(Debug, Clone, Default)]
pub struct PaletteState {
    /// Whether overlay is open.
    pub open: bool,
    /// Input query.
    pub query: String,
    /// Selected item index among filtered actions.
    pub selected: usize,
}

/// Render rows for the palette list view.
#[derive(Debug, Clone)]
pub enum PaletteRow {
    /// Section heading row.
    Section(PaletteSection),
    /// Horizontal separator row.
    Separator,
    /// Action row, value is source action index.
    Action(usize),
}

/// Filters actions by query and returns source indices.
pub fn filtered_action_indices(actions: &[PaletteAction], query: &str) -> Vec<usize> {
    let needle = query.trim().to_ascii_lowercase();
    if needle.is_empty() {
        return (0..actions.len()).collect();
    }

    let mut matches = Vec::new();
    for (index, action) in actions.iter().enumerate() {
        let title_match = action.title.to_ascii_lowercase().contains(&needle);
        let context_match = action.context.to_ascii_lowercase().contains(&needle);
        let keyword_match = action
            .keywords
            .iter()
            .any(|keyword| keyword.to_ascii_lowercase().contains(&needle));
        if title_match || context_match || keyword_match {
            matches.push(index);
        }
    }
    matches
}

/// Builds render rows with explicit separators between section buckets.
pub fn build_rows(actions: &[PaletteAction], filtered_indices: &[usize]) -> Vec<PaletteRow> {
    let mut context = Vec::new();
    let mut navigate = Vec::new();
    let mut global = Vec::new();

    for &index in filtered_indices {
        match actions[index].section {
            PaletteSection::Context => context.push(index),
            PaletteSection::Navigate => navigate.push(index),
            PaletteSection::Global => global.push(index),
        }
    }

    let mut rows = Vec::new();
    append_section_rows(&mut rows, PaletteSection::Context, &context);
    append_section_rows(&mut rows, PaletteSection::Navigate, &navigate);
    append_section_rows(&mut rows, PaletteSection::Global, &global);
    rows
}

fn append_section_rows(rows: &mut Vec<PaletteRow>, section: PaletteSection, entries: &[usize]) {
    if entries.is_empty() {
        return;
    }

    if !rows.is_empty() {
        rows.push(PaletteRow::Separator);
    }

    rows.push(PaletteRow::Section(section));
    rows.extend(entries.iter().map(|index| PaletteRow::Action(*index)));
}

/// Builds the `context` label for navigation entries.
pub fn workspace_context(workspace: Workspace) -> String {
    workspace.label().to_ascii_lowercase()
}
