//! Application state machine and event handling.

use std::collections::HashMap;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::Terminal;
use ratatui::backend::Backend;

use crate::api::{ApiEngine, FetchContext, RoutePayload};
use crate::palette::{
    PaletteAction, PaletteActionKind, PaletteSection, PaletteState, build_rows,
    filtered_action_indices, workspace_context,
};
use crate::routes::{RouteDefinition, RouteKind, build_routes};

/// Which pane is currently keyboard-focused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusTarget {
    /// Left navigation tree.
    Navigation,
    /// Main content pane.
    Main,
}

/// In-app input prompt type.
#[derive(Debug, Clone)]
pub enum PromptField {
    /// `bank_account` filter value.
    BankAccount,
    /// Self-assessment `user` value.
    SelfAssessmentUser,
    /// Payroll year.
    PayrollYear,
    /// Payroll period.
    PayrollPeriod,
    /// Target id/url for get-only routes.
    TargetId(String),
}

/// Selectable prompt option.
#[derive(Debug, Clone)]
pub struct PromptOption {
    /// Primary visible label.
    pub label: String,
    /// Canonical value written back to context.
    pub value: String,
    /// Secondary text shown to aid selection.
    pub meta: String,
}

/// Active prompt state.
#[derive(Debug, Clone)]
pub struct PromptState {
    /// Prompt title.
    pub title: String,
    /// Prompt helper text.
    pub hint: String,
    /// Current input value.
    pub value: String,
    /// Prompt target field.
    pub field: PromptField,
    /// Optional picker entries for this prompt.
    pub options: Vec<PromptOption>,
    /// Selected picker row.
    pub selected_option: usize,
}

/// Cached route view data.
#[derive(Debug, Clone, Default)]
pub struct RouteView {
    /// Last payload for the route.
    pub payload: Option<RoutePayload>,
    /// Last loading error.
    pub error: Option<String>,
    /// Row cursor for list payloads.
    pub selected_row: usize,
}

/// Runtime application.
pub struct App {
    /// API bridge.
    pub(crate) api: ApiEngine,
    /// Routes in deterministic display order.
    pub(crate) routes: Vec<RouteDefinition>,
    /// Active route index.
    pub(crate) active_route: usize,
    /// Navigation cursor index.
    pub(crate) nav_cursor: usize,
    /// Focus target.
    pub(crate) focus: FocusTarget,
    /// Whether left tree is visible.
    pub(crate) show_tree: bool,
    /// Palette state.
    pub(crate) palette: PaletteState,
    /// Prompt state when active.
    pub(crate) prompt: Option<PromptState>,
    /// Dynamic query context.
    pub(crate) context: FetchContext,
    /// Route cache keyed by route id.
    pub(crate) views: HashMap<String, RouteView>,
    /// Status line text.
    pub(crate) status: String,
    /// Max list limit.
    pub(crate) list_limit: usize,
    /// Palette action catalog for current open context.
    pub(crate) palette_actions: Vec<PaletteAction>,
    /// Filtered palette action indices.
    pub(crate) palette_filtered: Vec<usize>,
    /// Exit flag.
    pub(crate) should_quit: bool,
}

impl App {
    /// Creates and initializes the app.
    pub fn new() -> Result<Self, String> {
        let api = ApiEngine::new()?;
        let routes = build_routes();
        if routes.is_empty() {
            return Err("Route catalog is empty; cannot start cho-tui".to_string());
        }

        let list_limit = 100;
        let mut app = Self {
            api,
            routes,
            active_route: 0,
            nav_cursor: 0,
            focus: FocusTarget::Navigation,
            show_tree: true,
            palette: PaletteState::default(),
            prompt: None,
            context: FetchContext::default(),
            views: HashMap::new(),
            status: "Ready".to_string(),
            list_limit,
            palette_actions: Vec::new(),
            palette_filtered: Vec::new(),
            should_quit: false,
        };

        if !app.api.startup_warnings().is_empty() {
            app.status = app.api.startup_warnings().join(" | ");
        }

        app.load_active_route();
        Ok(app)
    }

    /// Main event/render loop.
    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<(), String> {
        while !self.should_quit {
            terminal
                .draw(|frame| crate::ui::render(frame, self))
                .map_err(|e| format!("render failed: {e}"))?;

            if !event::poll(Duration::from_millis(30))
                .map_err(|e| format!("event poll failed: {e}"))?
            {
                continue;
            }

            let event = event::read().map_err(|e| format!("event read failed: {e}"))?;
            if let Event::Key(key) = event
                && matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat)
            {
                self.handle_key_event(key);
            }
        }

        Ok(())
    }

    /// Returns currently active route.
    pub fn current_route(&self) -> &RouteDefinition {
        &self.routes[self.active_route]
    }

    /// Returns current route view data.
    pub fn current_view(&self) -> Option<&RouteView> {
        let route = self.current_route();
        self.views.get(&route.id)
    }

    /// Returns mutable current route view data.
    pub fn current_view_mut(&mut self) -> Option<&mut RouteView> {
        let route_id = self.current_route().id.clone();
        self.views.get_mut(&route_id)
    }

    fn handle_key_event(&mut self, key: KeyEvent) {
        if self.handle_prompt_key(key) {
            return;
        }
        if self.handle_palette_key(key) {
            return;
        }

        if is_palette_trigger(key) {
            self.open_palette();
            return;
        }

        match key.code {
            KeyCode::Char('q') => {
                self.should_quit = true;
                return;
            }
            KeyCode::Char('r') => {
                self.load_active_route();
                return;
            }
            KeyCode::Char('t') => {
                self.show_tree = !self.show_tree;
                return;
            }
            KeyCode::Tab => {
                self.focus = match self.focus {
                    FocusTarget::Navigation => FocusTarget::Main,
                    FocusTarget::Main => FocusTarget::Navigation,
                };
                return;
            }
            _ => {}
        }

        match self.focus {
            FocusTarget::Navigation => self.handle_nav_key(key),
            FocusTarget::Main => self.handle_main_key(key),
        }
    }

    fn handle_nav_key(&mut self, key: KeyEvent) {
        let previous_cursor = self.nav_cursor;

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.nav_cursor > 0 {
                    self.nav_cursor -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.nav_cursor + 1 < self.routes.len() {
                    self.nav_cursor += 1;
                }
            }
            KeyCode::PageUp => {
                self.nav_cursor = self.nav_cursor.saturating_sub(10);
            }
            KeyCode::PageDown => {
                let max_index = self.routes.len().saturating_sub(1);
                self.nav_cursor = (self.nav_cursor + 10).min(max_index);
            }
            KeyCode::Home => self.nav_cursor = 0,
            KeyCode::End => self.nav_cursor = self.routes.len().saturating_sub(1),
            KeyCode::Enter => {
                self.activate_navigation_selection(true);
            }
            _ => {}
        }

        if self.nav_cursor != previous_cursor {
            self.activate_navigation_selection(false);
        }
    }

    fn activate_navigation_selection(&mut self, force_reload: bool) {
        if self.active_route != self.nav_cursor {
            self.active_route = self.nav_cursor;
        } else if !force_reload {
            return;
        }

        self.load_active_route();
    }

    fn handle_main_key(&mut self, key: KeyEvent) {
        let len = self.current_list_len();
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if let Some(view) = self.current_view_mut()
                    && view.selected_row > 0
                {
                    view.selected_row -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if let Some(view) = self.current_view_mut()
                    && len > 0
                    && view.selected_row + 1 < len
                {
                    view.selected_row += 1;
                }
            }
            KeyCode::PageUp => {
                if let Some(view) = self.current_view_mut() {
                    view.selected_row = view.selected_row.saturating_sub(10);
                }
            }
            KeyCode::PageDown => {
                if let Some(view) = self.current_view_mut()
                    && len > 0
                {
                    view.selected_row = (view.selected_row + 10).min(len - 1);
                }
            }
            KeyCode::Home => {
                if let Some(view) = self.current_view_mut() {
                    view.selected_row = 0;
                }
            }
            KeyCode::End => {
                if let Some(view) = self.current_view_mut()
                    && len > 0
                {
                    view.selected_row = len - 1;
                }
            }
            KeyCode::Enter => {
                self.open_selected_item_detail();
            }
            _ => {}
        }
    }

    fn current_list_len(&self) -> usize {
        let Some(view) = self.current_view() else {
            return 0;
        };
        let Some(payload) = &view.payload else {
            return 0;
        };
        match payload {
            RoutePayload::List { items, .. } => items.len(),
            _ => 0,
        }
    }

    fn load_active_route(&mut self) {
        let route = self.current_route().clone();
        let entry = self.views.entry(route.id.clone()).or_default();
        entry.error = None;
        self.status = format!("Loading {}", route.label);

        match self.api.fetch_route(&route, &self.context, self.list_limit) {
            Ok(payload) => {
                entry.payload = Some(payload);
                entry.error = None;
                self.status = format!("Loaded {}", route.label);
                if let Some(RoutePayload::List { items, .. }) = entry.payload.as_ref() {
                    if items.is_empty() {
                        entry.selected_row = 0;
                    } else if entry.selected_row >= items.len() {
                        entry.selected_row = items.len().saturating_sub(1);
                    }
                }
            }
            Err(err) => {
                entry.error = Some(err.clone());
                entry.payload = None;
                self.status = format!("Error: {err}");
            }
        }
    }

    fn open_palette(&mut self) {
        self.palette.open = true;
        self.palette.query.clear();
        self.palette.selected = 0;
        self.rebuild_palette_actions();
    }

    fn close_palette(&mut self) {
        self.palette.open = false;
        self.palette.query.clear();
        self.palette.selected = 0;
        self.palette_filtered.clear();
        self.palette_actions.clear();
    }

    fn rebuild_palette_actions(&mut self) {
        self.palette_actions = self.build_palette_actions();
        self.palette_filtered = filtered_action_indices(&self.palette_actions, &self.palette.query);
        self.clamp_palette_selection();
    }

    fn clamp_palette_selection(&mut self) {
        if self.palette_filtered.is_empty() {
            self.palette.selected = 0;
            return;
        }
        if self.palette.selected >= self.palette_filtered.len() {
            self.palette.selected = self.palette_filtered.len().saturating_sub(1);
        }
    }

    fn build_palette_actions(&self) -> Vec<PaletteAction> {
        let mut actions = Vec::new();
        let route = self.current_route();

        actions.push(PaletteAction {
            title: "Refresh current page".to_string(),
            context: route.label.clone(),
            section: PaletteSection::Context,
            kind: PaletteActionKind::Refresh,
            keywords: vec!["reload".to_string(), "refresh".to_string()],
            disabled_reason: None,
        });

        match route.kind {
            RouteKind::Resource(spec)
                if spec.name == "bank-transactions"
                    || spec.name == "bank-transaction-explanations" =>
            {
                actions.push(PaletteAction {
                    title: "Set bank account filter".to_string(),
                    context: route.label.clone(),
                    section: PaletteSection::Context,
                    kind: PaletteActionKind::PromptBankAccount,
                    keywords: vec!["bank".to_string(), "filter".to_string()],
                    disabled_reason: None,
                });
            }
            RouteKind::SelfAssessmentReturns => actions.push(PaletteAction {
                title: "Set self-assessment user".to_string(),
                context: route.label.clone(),
                section: PaletteSection::Context,
                kind: PaletteActionKind::PromptSelfAssessmentUser,
                keywords: vec![
                    "self".to_string(),
                    "assessment".to_string(),
                    "user".to_string(),
                ],
                disabled_reason: None,
            }),
            RouteKind::PayrollPeriods | RouteKind::PayrollProfiles => actions.push(PaletteAction {
                title: "Set payroll year".to_string(),
                context: route.label.clone(),
                section: PaletteSection::Context,
                kind: PaletteActionKind::PromptPayrollYear,
                keywords: vec!["payroll".to_string(), "year".to_string()],
                disabled_reason: None,
            }),
            RouteKind::PayrollPeriodDetail => {
                actions.push(PaletteAction {
                    title: "Set payroll year".to_string(),
                    context: route.label.clone(),
                    section: PaletteSection::Context,
                    kind: PaletteActionKind::PromptPayrollYear,
                    keywords: vec!["payroll".to_string(), "year".to_string()],
                    disabled_reason: None,
                });
                actions.push(PaletteAction {
                    title: "Set payroll period".to_string(),
                    context: route.label.clone(),
                    section: PaletteSection::Context,
                    kind: PaletteActionKind::PromptPayrollPeriod,
                    keywords: vec!["payroll".to_string(), "period".to_string()],
                    disabled_reason: None,
                });
            }
            _ => {}
        }

        if let RouteKind::Resource(spec) = route.kind
            && !spec.capabilities.list
            && spec.capabilities.get
        {
            actions.push(PaletteAction {
                title: "Set target id".to_string(),
                context: route.label.clone(),
                section: PaletteSection::Context,
                kind: PaletteActionKind::PromptTargetId(route.id.clone()),
                keywords: vec!["id".to_string(), "url".to_string(), "target".to_string()],
                disabled_reason: None,
            });
        }

        actions.extend(self.build_disabled_write_actions(route));

        for route in &self.routes {
            actions.push(PaletteAction {
                title: route.label.clone(),
                context: workspace_context(route.workspace),
                section: PaletteSection::Navigate,
                kind: PaletteActionKind::Navigate(route.id.clone()),
                keywords: vec![route.id.clone(), route.workspace.label().to_string()],
                disabled_reason: None,
            });
        }

        actions.push(PaletteAction {
            title: if self.show_tree {
                "Hide navigation tree".to_string()
            } else {
                "Show navigation tree".to_string()
            },
            context: "layout".to_string(),
            section: PaletteSection::Global,
            kind: PaletteActionKind::ToggleTree,
            keywords: vec![
                "layout".to_string(),
                "tree".to_string(),
                "sidebar".to_string(),
            ],
            disabled_reason: None,
        });
        actions.push(PaletteAction {
            title: "Refresh".to_string(),
            context: "global".to_string(),
            section: PaletteSection::Global,
            kind: PaletteActionKind::Refresh,
            keywords: vec!["reload".to_string(), "refresh".to_string()],
            disabled_reason: None,
        });
        actions.push(PaletteAction {
            title: "Quit".to_string(),
            context: "global".to_string(),
            section: PaletteSection::Global,
            kind: PaletteActionKind::Quit,
            keywords: vec!["exit".to_string(), "quit".to_string()],
            disabled_reason: None,
        });

        actions
    }

    fn build_disabled_write_actions(&self, route: &RouteDefinition) -> Vec<PaletteAction> {
        let mut actions = Vec::new();
        let reason = "write actions disabled in read-only phase".to_string();

        if let RouteKind::Resource(spec) = route.kind {
            if spec.capabilities.create {
                actions.push(disabled_action("Create", route, reason.clone()));
            }
            if spec.capabilities.update {
                actions.push(disabled_action("Update selected", route, reason.clone()));
            }
            if spec.capabilities.delete {
                actions.push(disabled_action("Delete selected", route, reason.clone()));
            }
            if spec.name == "invoices" {
                for title in [
                    "Transition invoice: mark as draft",
                    "Transition invoice: mark as sent",
                    "Transition invoice: mark as scheduled",
                    "Transition invoice: mark as cancelled",
                    "Transition invoice: convert to credit note",
                    "Send invoice email",
                ] {
                    actions.push(disabled_action(title, route, reason.clone()));
                }
            }
            if spec.name == "bank-transactions" {
                actions.push(disabled_action("Upload statement", route, reason.clone()));
            }
            if matches!(
                spec.name,
                "vat-returns" | "corporation-tax-returns" | "final-accounts-reports"
            ) {
                for title in ["Mark filed", "Mark unfiled", "Mark paid", "Mark unpaid"] {
                    actions.push(disabled_action(title, route, reason.clone()));
                }
            }
        }

        if matches!(route.kind, RouteKind::SelfAssessmentReturns) {
            for title in [
                "Mark filed",
                "Mark unfiled",
                "Mark payment paid",
                "Mark payment unpaid",
            ] {
                actions.push(disabled_action(title, route, reason.clone()));
            }
        }

        if matches!(
            route.kind,
            RouteKind::PayrollPeriods | RouteKind::PayrollPeriodDetail | RouteKind::PayrollProfiles
        ) {
            for title in ["Mark payroll payment paid", "Mark payroll payment unpaid"] {
                actions.push(disabled_action(title, route, reason.clone()));
            }
        }

        actions
    }

    fn handle_palette_key(&mut self, key: KeyEvent) -> bool {
        if !self.palette.open {
            return false;
        }

        match key.code {
            KeyCode::Esc => self.close_palette(),
            KeyCode::Up => {
                if self.palette.selected > 0 {
                    self.palette.selected -= 1;
                }
            }
            KeyCode::Down => {
                if self.palette.selected + 1 < self.palette_filtered.len() {
                    self.palette.selected += 1;
                }
            }
            KeyCode::PageUp => {
                self.palette.selected = self.palette.selected.saturating_sub(10);
            }
            KeyCode::PageDown => {
                if self.palette_filtered.is_empty() {
                    self.palette.selected = 0;
                } else {
                    let max_index = self.palette_filtered.len() - 1;
                    self.palette.selected = (self.palette.selected + 10).min(max_index);
                }
            }
            KeyCode::Home => self.palette.selected = 0,
            KeyCode::End => {
                if self.palette_filtered.is_empty() {
                    self.palette.selected = 0;
                } else {
                    self.palette.selected = self.palette_filtered.len() - 1;
                }
            }
            KeyCode::Enter => self.execute_selected_palette_action(),
            KeyCode::Backspace => {
                self.palette.query.pop();
                self.rebuild_palette_actions();
            }
            KeyCode::Char(ch) => {
                if key.modifiers.contains(KeyModifiers::CONTROL)
                    || key.modifiers.contains(KeyModifiers::SUPER)
                {
                    return true;
                }
                self.palette.query.push(ch);
                self.rebuild_palette_actions();
            }
            _ => {}
        }

        true
    }

    fn execute_selected_palette_action(&mut self) {
        let Some(source_index) = self.palette_filtered.get(self.palette.selected).copied() else {
            return;
        };
        let Some(action) = self.palette_actions.get(source_index).cloned() else {
            return;
        };

        if let Some(reason) = action.disabled_reason {
            self.status = reason;
            return;
        }

        match action.kind {
            PaletteActionKind::Navigate(route_id) => {
                if let Some(index) = self.routes.iter().position(|route| route.id == route_id) {
                    self.active_route = index;
                    self.nav_cursor = index;
                    self.close_palette();
                    self.load_active_route();
                }
            }
            PaletteActionKind::Refresh => {
                self.close_palette();
                self.load_active_route();
            }
            PaletteActionKind::Quit => self.should_quit = true,
            PaletteActionKind::ToggleTree => {
                self.show_tree = !self.show_tree;
                self.close_palette();
            }
            PaletteActionKind::PromptBankAccount => {
                self.open_bank_account_prompt();
                self.close_palette();
            }
            PaletteActionKind::PromptSelfAssessmentUser => {
                self.open_self_assessment_user_prompt();
                self.close_palette();
            }
            PaletteActionKind::PromptPayrollYear => {
                self.prompt = Some(PromptState {
                    title: "Set Payroll Year".to_string(),
                    hint: "Example: 2026".to_string(),
                    value: self.context.payroll_year.to_string(),
                    field: PromptField::PayrollYear,
                    options: Vec::new(),
                    selected_option: 0,
                });
                self.close_palette();
            }
            PaletteActionKind::PromptPayrollPeriod => {
                self.prompt = Some(PromptState {
                    title: "Set Payroll Period".to_string(),
                    hint: "Example: 1..12".to_string(),
                    value: self.context.payroll_period.to_string(),
                    field: PromptField::PayrollPeriod,
                    options: Vec::new(),
                    selected_option: 0,
                });
                self.close_palette();
            }
            PaletteActionKind::PromptTargetId(route_id) => {
                let current = self
                    .context
                    .resource_targets
                    .get(&route_id)
                    .cloned()
                    .unwrap_or_default();
                self.prompt = Some(PromptState {
                    title: "Set Target ID".to_string(),
                    hint: "Paste ID or full resource URL".to_string(),
                    value: current,
                    field: PromptField::TargetId(route_id),
                    options: Vec::new(),
                    selected_option: 0,
                });
                self.close_palette();
            }
            PaletteActionKind::DisabledWriteAction => {}
        }
    }

    fn handle_prompt_key(&mut self, key: KeyEvent) -> bool {
        let Some(prompt) = self.prompt.as_mut() else {
            return false;
        };

        match key.code {
            KeyCode::Esc => {
                self.prompt = None;
                self.status = "Prompt cancelled".to_string();
            }
            KeyCode::Up => {
                if !prompt.options.is_empty() {
                    prompt.selected_option = prompt.selected_option.saturating_sub(1);
                }
            }
            KeyCode::Down => {
                if prompt.selected_option + 1 < prompt.options.len() {
                    prompt.selected_option += 1;
                }
            }
            KeyCode::PageUp => {
                prompt.selected_option = prompt.selected_option.saturating_sub(10);
            }
            KeyCode::PageDown => {
                if prompt.options.is_empty() {
                    prompt.selected_option = 0;
                } else {
                    let max_index = prompt.options.len() - 1;
                    prompt.selected_option = (prompt.selected_option + 10).min(max_index);
                }
            }
            KeyCode::Home => prompt.selected_option = 0,
            KeyCode::End => {
                prompt.selected_option = prompt.options.len().saturating_sub(1);
            }
            KeyCode::Enter => {
                let value = prompt_submit_value(prompt);
                let field = prompt.field.clone();
                self.prompt = None;
                self.apply_prompt(field, value);
            }
            KeyCode::Backspace => {
                prompt.value.pop();
                if !prompt.options.is_empty() {
                    select_prompt_option_from_query(prompt);
                }
            }
            KeyCode::Char(ch) => {
                if key.modifiers.contains(KeyModifiers::CONTROL)
                    || key.modifiers.contains(KeyModifiers::SUPER)
                {
                    return true;
                }
                prompt.value.push(ch);
                if !prompt.options.is_empty() {
                    select_prompt_option_from_query(prompt);
                }
            }
            _ => {}
        }

        true
    }

    fn apply_prompt(&mut self, field: PromptField, value: String) {
        match field {
            PromptField::BankAccount => {
                if value.is_empty() {
                    self.context.bank_account_filter = None;
                    self.status = "Cleared bank account filter".to_string();
                } else {
                    self.context.bank_account_filter = Some(value);
                    self.status = "Updated bank account filter".to_string();
                }
                self.load_active_route();
            }
            PromptField::SelfAssessmentUser => {
                if value.is_empty() {
                    self.context.self_assessment_user = None;
                    self.status = "Cleared self-assessment user".to_string();
                } else {
                    self.context.self_assessment_user = Some(value);
                    self.status = "Updated self-assessment user".to_string();
                }
                self.load_active_route();
            }
            PromptField::PayrollYear => match value.parse::<i32>() {
                Ok(year) => {
                    self.context.payroll_year = year;
                    self.status = format!("Set payroll year to {year}");
                    self.load_active_route();
                }
                Err(_) => self.status = format!("Invalid payroll year '{value}'"),
            },
            PromptField::PayrollPeriod => match value.parse::<i32>() {
                Ok(period) => {
                    self.context.payroll_period = period;
                    self.status = format!("Set payroll period to {period}");
                    self.load_active_route();
                }
                Err(_) => self.status = format!("Invalid payroll period '{value}'"),
            },
            PromptField::TargetId(route_id) => {
                if value.is_empty() {
                    self.context.resource_targets.remove(&route_id);
                    self.status = "Cleared target id".to_string();
                } else {
                    self.context.resource_targets.insert(route_id, value);
                    self.status = "Updated target id".to_string();
                }
                self.load_active_route();
            }
        }
    }

    fn open_selected_item_detail(&mut self) {
        let route = self.current_route().clone();
        let Some(view) = self.current_view() else {
            return;
        };
        let Some(RoutePayload::List { items, .. }) = &view.payload else {
            return;
        };
        if items.is_empty() {
            return;
        }
        let selected = view.selected_row.min(items.len().saturating_sub(1));
        let item = items[selected].clone();

        match route.kind {
            RouteKind::Resource(spec) if spec.capabilities.get => {
                let Some(id) = infer_item_identifier(&item) else {
                    self.status = "Selected row does not contain id/url".to_string();
                    return;
                };
                match self.api.fetch_resource_item(spec, &id) {
                    Ok(payload) => {
                        let entry = self.views.entry(route.id.clone()).or_default();
                        entry.payload = Some(payload);
                        entry.error = None;
                        self.status = format!("Loaded {}", route.label);
                    }
                    Err(err) => self.status = err,
                }
            }
            RouteKind::SelfAssessmentReturns => {
                let Some(user) = self
                    .context
                    .self_assessment_user
                    .as_ref()
                    .filter(|value| !value.trim().is_empty())
                    .cloned()
                else {
                    self.status = "Set self-assessment user first".to_string();
                    return;
                };
                let period = item
                    .get("period_ends_on")
                    .and_then(|value| value.as_str())
                    .map(str::to_string)
                    .or_else(|| {
                        item.get("period_end")
                            .and_then(|value| value.as_str())
                            .map(str::to_string)
                    });
                let Some(period_ends_on) = period else {
                    self.status = "Selected row missing period_ends_on".to_string();
                    return;
                };
                match self.api.fetch_self_assessment_item(&user, &period_ends_on) {
                    Ok(payload) => {
                        let entry = self.views.entry(route.id.clone()).or_default();
                        entry.payload = Some(payload);
                        entry.error = None;
                        self.status = format!("Loaded {}", route.label);
                    }
                    Err(err) => self.status = err,
                }
            }
            _ => {}
        }
    }

    /// Palette rows for rendering with section separators.
    pub fn palette_rows(&self) -> Vec<crate::palette::PaletteRow> {
        build_rows(&self.palette_actions, &self.palette_filtered)
    }

    fn open_bank_account_prompt(&mut self) {
        let options = self.load_prompt_options("bank-accounts", bank_account_prompt_option);
        let mut prompt = PromptState {
            title: "Set Bank Account Filter".to_string(),
            hint: if options.is_empty() {
                "No accounts loaded. Paste bank account URL manually".to_string()
            } else {
                "Type to jump, arrows to select account, enter to confirm".to_string()
            },
            value: self.context.bank_account_filter.clone().unwrap_or_default(),
            field: PromptField::BankAccount,
            options,
            selected_option: 0,
        };
        sync_prompt_selection_from_value(&mut prompt);
        self.prompt = Some(prompt);
    }

    fn open_self_assessment_user_prompt(&mut self) {
        let options = self.load_prompt_options("users", user_prompt_option);
        let mut prompt = PromptState {
            title: "Set Self-Assessment User".to_string(),
            hint: if options.is_empty() {
                "No users loaded. Paste user ID or user URL manually".to_string()
            } else {
                "Type to jump, arrows to select user, enter to confirm".to_string()
            },
            value: self
                .context
                .self_assessment_user
                .clone()
                .unwrap_or_default(),
            field: PromptField::SelfAssessmentUser,
            options,
            selected_option: 0,
        };
        sync_prompt_selection_from_value(&mut prompt);
        self.prompt = Some(prompt);
    }

    fn load_prompt_options(
        &mut self,
        route_id: &str,
        mapper: fn(&serde_json::Value) -> Option<PromptOption>,
    ) -> Vec<PromptOption> {
        if let Some(view) = self.views.get(route_id)
            && let Some(RoutePayload::List { items, .. }) = &view.payload
        {
            return items.iter().filter_map(mapper).collect();
        }

        let Some(route) = self
            .routes
            .iter()
            .find(|route| route.id == route_id)
            .cloned()
        else {
            return Vec::new();
        };

        match self.api.fetch_route(&route, &self.context, self.list_limit) {
            Ok(RoutePayload::List { items, .. }) => items.iter().filter_map(mapper).collect(),
            Ok(RoutePayload::Message(message)) => {
                self.status = message;
                Vec::new()
            }
            Ok(_) => Vec::new(),
            Err(err) => {
                self.status = err;
                Vec::new()
            }
        }
    }
}

fn disabled_action(title: &str, route: &RouteDefinition, reason: String) -> PaletteAction {
    PaletteAction {
        title: title.to_string(),
        context: route.label.clone(),
        section: PaletteSection::Context,
        kind: PaletteActionKind::DisabledWriteAction,
        keywords: vec!["write".to_string(), "mutate".to_string(), title.to_string()],
        disabled_reason: Some(reason),
    }
}

fn is_palette_trigger(key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Char(ch) => {
            ch.eq_ignore_ascii_case(&'p')
                && (key.modifiers.contains(KeyModifiers::SUPER)
                    || key.modifiers.contains(KeyModifiers::CONTROL))
        }
        _ => false,
    }
}

fn infer_item_identifier(value: &serde_json::Value) -> Option<String> {
    if let Some(url) = value.get("url").and_then(|item| item.as_str()) {
        return Some(url.to_string());
    }

    if let Some(id) = value.get("id").and_then(|item| item.as_str()) {
        return Some(id.to_string());
    }

    None
}

fn prompt_submit_value(prompt: &PromptState) -> String {
    let typed = prompt.value.trim();
    if prompt.options.is_empty() {
        return typed.to_string();
    }

    if typed.is_empty() {
        return prompt
            .options
            .get(prompt.selected_option)
            .map(|option| option.value.clone())
            .unwrap_or_default();
    }

    if prompt.options.iter().any(|option| option.value == typed) {
        return typed.to_string();
    }

    let query = typed.to_ascii_lowercase();
    if let Some(option) = prompt.options.get(prompt.selected_option)
        && prompt_option_matches(option, &query)
    {
        return option.value.clone();
    }

    typed.to_string()
}

fn sync_prompt_selection_from_value(prompt: &mut PromptState) {
    if prompt.options.is_empty() {
        prompt.selected_option = 0;
        return;
    }

    let value = prompt.value.trim();
    if value.is_empty() {
        prompt.selected_option = 0;
        return;
    }

    if let Some(index) = prompt
        .options
        .iter()
        .position(|option| option.value == value)
    {
        prompt.selected_option = index;
        return;
    }

    select_prompt_option_from_query(prompt);
}

fn select_prompt_option_from_query(prompt: &mut PromptState) {
    if prompt.options.is_empty() {
        prompt.selected_option = 0;
        return;
    }

    let query = prompt.value.trim().to_ascii_lowercase();
    if query.is_empty() {
        prompt.selected_option = 0;
        return;
    }

    if let Some((index, _)) = prompt
        .options
        .iter()
        .enumerate()
        .find(|(_, option)| prompt_option_matches(option, &query))
    {
        prompt.selected_option = index;
    }
}

fn prompt_option_matches(option: &PromptOption, query: &str) -> bool {
    option.label.to_ascii_lowercase().contains(query)
        || option.meta.to_ascii_lowercase().contains(query)
        || option.value.to_ascii_lowercase().contains(query)
}

fn bank_account_prompt_option(item: &serde_json::Value) -> Option<PromptOption> {
    let value = infer_item_identifier(item)?;
    let name = item
        .get("name")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("Bank Account");
    let bank_name = item
        .get("bank_name")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    let account_number = item
        .get("account_number")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();

    let mut meta_parts = Vec::new();
    if !bank_name.is_empty() {
        meta_parts.push(bank_name.to_string());
    }
    if !account_number.is_empty() {
        meta_parts.push(account_number.to_string());
    }

    Some(PromptOption {
        label: name.to_string(),
        value,
        meta: meta_parts.join(" | "),
    })
}

fn user_prompt_option(item: &serde_json::Value) -> Option<PromptOption> {
    let value = infer_item_identifier(item)?;
    let first_name = item
        .get("first_name")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    let last_name = item
        .get("last_name")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    let email = item
        .get("email")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    let role = item
        .get("role")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();

    let full_name = format!("{first_name} {last_name}").trim().to_string();
    let label = if !full_name.is_empty() {
        full_name
    } else if !email.is_empty() {
        email.to_string()
    } else {
        "User".to_string()
    };

    let mut meta_parts = Vec::new();
    if !role.is_empty() {
        meta_parts.push(role.to_string());
    }
    if !email.is_empty() {
        meta_parts.push(email.to_string());
    }

    Some(PromptOption {
        label,
        value,
        meta: meta_parts.join(" | "),
    })
}
