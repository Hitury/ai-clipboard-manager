mod app;
mod category;
mod history;
mod theme;
mod ui;
mod watcher;

use app::App;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyEventKind},
};
use std::{io, sync::mpsc::Receiver, time::Duration};

/// How long to wait for input before checking the clipboard and redrawing (keeps ages fresh).
const TICK: Duration = Duration::from_millis(250);

fn main() -> io::Result<()> {
    let clips = watcher::spawn();
    let mut app = App::new();
    ratatui::run(|terminal| run(terminal, &mut app, &clips))
}

fn run(terminal: &mut DefaultTerminal, app: &mut App, clips: &Receiver<String>) -> io::Result<()> {
    while app.running {
        terminal.draw(|f| ui::draw(f, app))?;
        if event::poll(TICK)?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            app.on_key(key);
        }
        clips.try_iter().for_each(|text| app.on_clip(text));
        app.persist();
    }
    Ok(())
}
