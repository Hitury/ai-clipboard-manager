use crate::{
    category::Category,
    history::{Entry, History},
};
use arboard::Clipboard;
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    widgets::TableState,
};

const CLEAR_PROMPT: &str = "Press C again to clear all unpinned entries";

#[derive(Clone, Copy, PartialEq)]
pub enum Tab {
    All,
    Pinned,
    Kind(Category),
}

pub const TABS: [Tab; 9] = {
    use Category::*;
    [
        Tab::All,
        Tab::Pinned,
        Tab::Kind(Text),
        Tab::Kind(Link),
        Tab::Kind(Code),
        Tab::Kind(Path),
        Tab::Kind(Email),
        Tab::Kind(Color),
        Tab::Kind(Number),
    ]
};

impl Tab {
    pub fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Pinned => "Pinned",
            Self::Kind(c) => c.label(),
        }
    }

    pub fn matches(self, entry: &Entry) -> bool {
        match self {
            Self::All => true,
            Self::Pinned => entry.pinned,
            Self::Kind(c) => entry.category == c,
        }
    }
}

#[derive(PartialEq)]
pub enum Mode {
    Normal,
    Search,
}

pub struct App {
    pub history: History,
    pub tab: usize,
    pub mode: Mode,
    pub query: String,
    pub status: &'static str,
    pub table: TableState,
    /// Indices into `history.entries` that pass the current tab and search filter.
    pub visible: Vec<usize>,
    pub running: bool,
    dirty: bool,
    clipboard: Option<Clipboard>,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            history: History::load(),
            tab: 0,
            mode: Mode::Normal,
            query: String::new(),
            status: "",
            table: TableState::default(),
            visible: Vec::new(),
            running: true,
            dirty: false,
            clipboard: Clipboard::new().ok(),
        };
        app.refresh();
        app
    }

    pub fn selected(&self) -> Option<&Entry> {
        let i = *self.visible.get(self.table.selected()?)?;
        self.history.entries.get(i)
    }

    pub fn on_clip(&mut self, text: String) {
        self.dirty |= self.history.add(text);
        self.refresh();
    }

    pub fn on_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.running = false;
            return;
        }
        let prev_status = std::mem::take(&mut self.status);
        match self.mode {
            Mode::Search => self.search_key(key.code),
            Mode::Normal => self.normal_key(key.code, prev_status),
        }
        self.refresh();
    }

    /// Writes history to disk if it changed since the last save.
    pub fn persist(&mut self) {
        if std::mem::take(&mut self.dirty) && self.history.save().is_err() {
            self.status = "Failed to save history";
        }
    }

    fn search_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc => {
                self.query.clear();
                self.mode = Mode::Normal;
            }
            KeyCode::Enter => self.mode = Mode::Normal,
            KeyCode::Up => self.table.select_previous(),
            KeyCode::Down => self.table.select_next(),
            KeyCode::Backspace => {
                self.query.pop();
                self.table.select_first();
            }
            KeyCode::Char(c) => {
                self.query.push(c);
                self.table.select_first();
            }
            _ => {}
        }
    }

    fn normal_key(&mut self, code: KeyCode, prev_status: &str) {
        match code {
            KeyCode::Char('q') => self.running = false,
            KeyCode::Esc if self.query.is_empty() => self.running = false,
            KeyCode::Esc => self.query.clear(),
            KeyCode::Char('/') => self.mode = Mode::Search,
            KeyCode::Char('j') | KeyCode::Down => self.table.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.table.select_previous(),
            KeyCode::Char('g') | KeyCode::Home => self.table.select_first(),
            KeyCode::Char('G') | KeyCode::End => self.table.select_last(),
            KeyCode::Char('l') | KeyCode::Right | KeyCode::Tab => self.switch_tab(1),
            KeyCode::Char('h') | KeyCode::Left | KeyCode::BackTab => self.switch_tab(TABS.len() - 1),
            KeyCode::Enter | KeyCode::Char('y') => self.copy(),
            KeyCode::Char('p') => self.edit_selected(|h, i| h.entries[i].pinned ^= true),
            KeyCode::Char('d') | KeyCode::Delete => self.edit_selected(|h, i| _ = h.entries.remove(i)),
            KeyCode::Char('C') if prev_status == CLEAR_PROMPT => {
                self.history.clear();
                self.dirty = true;
            }
            KeyCode::Char('C') => self.status = CLEAR_PROMPT,
            _ => {}
        }
    }

    fn switch_tab(&mut self, step: usize) {
        self.tab = (self.tab + step) % TABS.len();
        self.table.select_first();
    }

    fn edit_selected(&mut self, edit: impl FnOnce(&mut History, usize)) {
        if let Some(&i) = self.table.selected().and_then(|s| self.visible.get(s)) {
            edit(&mut self.history, i);
            self.dirty = true;
        }
    }

    /// Copies the selected entry back to the system clipboard and moves it to the top.
    fn copy(&mut self) {
        let Some(text) = self.selected().map(|e| e.content.clone()) else {
            return;
        };
        let Some(clipboard) = self.clipboard.as_mut() else {
            self.status = "Clipboard unavailable";
            return;
        };
        self.status = match clipboard.set_text(text.as_str()) {
            Ok(()) => "Copied to clipboard",
            Err(_) => "Failed to copy",
        };
        self.dirty |= self.history.add(text);
        self.table.select_first();
    }

    /// Recomputes the filtered view and keeps the selection in bounds.
    fn refresh(&mut self) {
        let tab = TABS[self.tab];
        let query = self.query.to_lowercase();
        self.visible = (self.history.entries.iter().enumerate())
            .filter(|(_, e)| {
                tab.matches(e) && (query.is_empty() || e.content.to_lowercase().contains(&query))
            })
            .map(|(i, _)| i)
            .collect();
        let last = self.visible.len().checked_sub(1);
        self.table.select(last.map(|last| self.table.selected().unwrap_or(0).min(last)));
    }
}
