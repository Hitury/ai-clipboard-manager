use crate::category::Category;
use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

/// Maximum number of stored entries; the oldest unpinned ones are evicted first.
const LIMIT: usize = 500;

#[derive(Clone, Serialize, Deserialize)]
pub struct Entry {
    pub content: String,
    pub category: Category,
    pub pinned: bool,
    pub time: u64,
}

impl Entry {
    fn new(content: String) -> Self {
        Self { category: Category::detect(&content), content, pinned: false, time: now() }
    }

    /// Compact relative age, e.g. `42s`, `5m`, `3h`, `2d`.
    pub fn age(&self) -> String {
        match now().saturating_sub(self.time) {
            s @ 0..60 => format!("{s}s"),
            s @ 60..3600 => format!("{}m", s / 60),
            s @ 3600..86400 => format!("{}h", s / 3600),
            s => format!("{}d", s / 86400),
        }
    }
}

/// Clipboard history, newest entry first.
#[derive(Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct History {
    pub entries: Vec<Entry>,
}

impl History {
    pub fn load() -> Self {
        fs::read_to_string(path()).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
    }

    pub fn save(&self) -> io::Result<()> {
        let path = path();
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        fs::write(path, serde_json::to_string(self)?)
    }

    /// Puts `content` on top, reusing an existing duplicate. Returns false if nothing changed.
    pub fn add(&mut self, content: String) -> bool {
        if self.entries.first().is_some_and(|e| e.content == content) {
            return false;
        }
        let entry = match self.entries.iter().position(|e| e.content == content) {
            Some(i) => Entry { time: now(), ..self.entries.remove(i) },
            None => Entry::new(content),
        };
        self.entries.insert(0, entry);
        if self.entries.len() > LIMIT
            && let Some(i) = self.entries.iter().rposition(|e| !e.pinned)
        {
            self.entries.remove(i);
        }
        true
    }

    /// Removes every entry that isn't pinned.
    pub fn clear(&mut self) {
        self.entries.retain(|e| e.pinned);
    }
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

fn path() -> PathBuf {
    dirs::data_dir().unwrap_or_default().join("clipboard-manager").join("history.json")
}
