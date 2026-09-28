use std::{
    sync::mpsc::{self, Receiver},
    thread,
    time::Duration,
};

const POLL_INTERVAL: Duration = Duration::from_millis(400);

/// Polls the system clipboard on a background thread and yields each new text value.
pub fn spawn() -> Receiver<String> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let Ok(mut clipboard) = arboard::Clipboard::new() else {
            return;
        };
        let mut last = String::new();
        loop {
            if let Ok(text) = clipboard.get_text()
                && text != last
                && !text.trim().is_empty()
            {
                last.clone_from(&text);
                if tx.send(text).is_err() {
                    break;
                }
            }
            thread::sleep(POLL_INTERVAL);
        }
    });
    rx
}
