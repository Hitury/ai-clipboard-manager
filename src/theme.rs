//! Slate palette (Tailwind CSS shades) with one muted accent per category.

use crate::{app::Tab, category::Category};
use ratatui::style::Color;

const fn hex(rgb: u32) -> Color {
    Color::Rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

pub const BG: Color = hex(0x0f172a); // slate-900
pub const SURFACE: Color = hex(0x1e293b); // slate-800
pub const BORDER: Color = hex(0x334155); // slate-700
pub const DIM: Color = hex(0x64748b); // slate-500
pub const MUTED: Color = hex(0x94a3b8); // slate-400
pub const TEXT: Color = hex(0xe2e8f0); // slate-200
pub const ACCENT: Color = hex(0x38bdf8); // sky-400
pub const PIN: Color = hex(0xfcd34d); // amber-300

pub fn category(c: Category) -> Color {
    match c {
        Category::Text => hex(0xcbd5e1),   // slate-300
        Category::Link => hex(0x818cf8),   // indigo-400
        Category::Code => hex(0x34d399),   // emerald-400
        Category::Path => hex(0xfb923c),   // orange-400
        Category::Email => hex(0xe879f9),  // fuchsia-400
        Category::Color => hex(0xfb7185),  // rose-400
        Category::Number => hex(0xa3e635), // lime-400
    }
}

/// Sidebar glyph and color for a tab.
pub fn tab(tab: Tab) -> (&'static str, Color) {
    match tab {
        Tab::All => ("◆", ACCENT),
        Tab::Pinned => ("★", PIN),
        Tab::Kind(c) => ("●", category(c)),
    }
}
