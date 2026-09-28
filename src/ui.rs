use crate::{
    app::{App, Mode, TABS},
    category::Category,
    theme::{self, ACCENT, BG, BORDER, DIM, MUTED, PIN, SURFACE, TEXT},
};
use ratatui::{
    Frame,
    layout::{Constraint::*, Layout, Position, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Cell, List, ListState, Padding, Paragraph, Row, Table, Wrap},
};

/// Max characters rendered for a row snippet or preview, keeping huge clips cheap to draw.
const SNIPPET_LEN: usize = 256;
const PREVIEW_LEN: usize = 8192;

pub fn draw(f: &mut Frame, app: &mut App) {
    f.render_widget(Block::new().bg(BG).fg(TEXT), f.area());
    let [top, body, bottom] = Layout::vertical([Length(3), Fill(1), Length(1)]).areas(f.area());
    let [side, list, detail] = Layout::horizontal([Length(22), Fill(3), Fill(2)]).areas(body);
    search_bar(f, app, top);
    sidebar(f, app, side);
    entries(f, app, list);
    preview(f, app, detail);
    footer(f, app, bottom);
}

fn panel(title: &str, focused: bool) -> Block<'static> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(if focused { ACCENT } else { BORDER })
        .title(format!(" {title} ").fg(if focused { TEXT } else { MUTED }))
        .padding(Padding::horizontal(1))
}

fn search_bar(f: &mut Frame, app: &App, area: Rect) {
    let searching = app.mode == Mode::Search;
    let text = match (app.query.is_empty(), searching) {
        (true, false) => Span::from("press / to search").fg(DIM),
        _ => Span::from(app.query.as_str()).fg(TEXT),
    };
    let block = panel("◆ clipboard", searching)
        .title(Line::from(format!(" {} items ", app.history.entries.len())).right_aligned().fg(DIM));
    f.render_widget(Paragraph::new(Line::from(vec!["⌕ ".fg(MUTED), text])).block(block), area);
    if searching {
        let x = area.x + 4 + app.query.chars().count() as u16;
        f.set_cursor_position(Position::new(x.min(area.right() - 2), area.y + 1));
    }
}

fn sidebar(f: &mut Frame, app: &App, area: Rect) {
    let items = TABS.iter().map(|&tab| {
        let (glyph, color) = theme::tab(tab);
        let count = app.history.entries.iter().filter(|e| tab.matches(e)).count();
        Line::from(vec![
            format!("{glyph} ").fg(color),
            format!("{:<12}", tab.label()).into(),
            format!("{count:>4} ").fg(DIM),
        ])
    });
    let list = List::new(items)
        .fg(MUTED)
        .highlight_style(Style::new().bg(SURFACE).fg(TEXT).bold())
        .highlight_symbol("▌")
        .block(panel("Categories", false).padding(Padding::ZERO));
    f.render_stateful_widget(list, area, &mut ListState::default().with_selected(Some(app.tab)));
}

fn entries(f: &mut Frame, app: &mut App, area: Rect) {
    let rows = app.visible.iter().map(|&i| {
        let e = &app.history.entries[i];
        let mut lines = e.content.trim().lines();
        let first: String = lines.next().unwrap_or_default().trim().chars().take(SNIPPET_LEN).collect();
        let more = lines.count();
        let snippet = Line::from(vec![
            first.replace('\t', "  ").into(),
            if more > 0 { format!("  +{more}").fg(DIM) } else { Span::default() },
        ]);
        Row::new([
            Cell::from("●".fg(theme::category(e.category))),
            Cell::from(snippet),
            Cell::from(if e.pinned { "★".fg(PIN) } else { Span::default() }),
            Cell::from(Line::from(e.age()).right_aligned().fg(DIM)),
        ])
    });
    let title = format!("{} · {}", TABS[app.tab].label(), app.visible.len());
    let table = Table::new(rows, [Length(1), Fill(1), Length(1), Length(3)])
        .row_highlight_style(Style::new().bg(SURFACE).fg(TEXT).bold())
        .block(panel(&title, app.mode == Mode::Normal));
    f.render_stateful_widget(table, area, &mut app.table);
}

fn preview(f: &mut Frame, app: &App, area: Rect) {
    let block = panel("Preview", false);
    let Some(e) = app.selected() else {
        let hint =
            if app.history.entries.is_empty() { "Copy something to get started" } else { "No matches" };
        f.render_widget(Paragraph::new(hint.fg(DIM)).centered().block(block), area);
        return;
    };
    let [meta, _, body] = Layout::vertical([Length(2), Length(1), Fill(1)]).areas(block.inner(area));
    f.render_widget(block, area);

    let mut kind = vec![format!("● {}", e.category.label()).fg(theme::category(e.category))];
    if e.category == Category::Color
        && let Ok(swatch) = e.content.trim().parse::<Color>()
    {
        kind.extend(["  ".into(), "    ".bg(swatch)]);
    }
    let (chars, lines) = (e.content.chars().count(), e.content.lines().count());
    let stats = format!("{chars} chars · {lines} ln · {} ago", e.age()).fg(DIM);
    f.render_widget(Paragraph::new(vec![Line::from(kind), stats.into()]), meta);

    let text: String = e.content.chars().take(PREVIEW_LEN).collect();
    f.render_widget(Paragraph::new(text.replace('\t', "    ")).wrap(Wrap { trim: false }), body);
}

fn footer(f: &mut Frame, app: &App, area: Rect) {
    if !app.status.is_empty() {
        return f.render_widget(Line::from(format!(" {}", app.status)).fg(ACCENT), area);
    }
    let keys: &[(&str, &str)] = match app.mode {
        Mode::Normal => &[
            ("↑↓", "move"),
            ("←→", "category"),
            ("⏎", "copy"),
            ("p", "pin"),
            ("d", "delete"),
            ("/", "search"),
            ("C", "clear"),
            ("q", "quit"),
        ],
        Mode::Search => &[("type", "filter"), ("↑↓", "move"), ("⏎", "done"), ("esc", "cancel")],
    };
    let spans = keys
        .iter()
        .flat_map(|(key, action)| [format!(" {key} ").fg(TEXT).bg(SURFACE), format!(" {action}  ").fg(DIM)]);
    f.render_widget(Line::from_iter(spans), area);
}
