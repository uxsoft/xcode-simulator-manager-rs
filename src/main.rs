mod app;
mod events;
mod simctl;
mod sizing;
mod ui;

use std::sync::mpsc::{Receiver, TryRecvError};
use std::thread;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event};
use ratatui::DefaultTerminal;

use crate::app::{App, Modal};
use crate::events::Action;
use crate::simctl::Udid;
use crate::sizing::SizeUpdate;

const TICK: Duration = Duration::from_millis(80);

enum DeleteOutcome {
    Done(Vec<Udid>),
    Failed(String),
}

fn main() -> Result<()> {
    let mut terminal = ratatui::init();
    let result = run(&mut terminal);
    ratatui::restore();
    result
}

fn run(terminal: &mut DefaultTerminal) -> Result<()> {
    let sims = simctl::list_devices()?;
    let mut app = App::new(sims);
    let mut size_rx = start_size_scan(&app);
    let mut delete_rx: Option<Receiver<DeleteOutcome>> = None;

    while !app.should_quit {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        // Drain background size updates.
        let mut got_any = false;
        loop {
            match size_rx.try_recv() {
                Ok(SizeUpdate { udid, bytes }) => {
                    app.apply_size(&udid, bytes);
                    got_any = true;
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    app.scanning = false;
                    break;
                }
            }
        }
        if got_any {
            app.resort();
        }

        // Drain delete outcome if a deletion is in flight.
        if let Some(rx) = &delete_rx {
            match rx.try_recv() {
                Ok(DeleteOutcome::Done(udids)) => {
                    app.remove_deleted(&udids);
                    app.modal = Modal::None;
                    delete_rx = None;
                }
                Ok(DeleteOutcome::Failed(msg)) => {
                    app.modal = Modal::Error(msg);
                    delete_rx = None;
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    delete_rx = None;
                    if matches!(app.modal, Modal::Deleting) {
                        app.modal = Modal::None;
                    }
                }
            }
        }

        if !event::poll(TICK)? {
            continue;
        }
        let Event::Key(key) = event::read()? else {
            continue;
        };

        match events::translate(&app.modal, key) {
            Action::Quit => app.should_quit = true,
            Action::MoveUp => app.move_cursor(-1),
            Action::MoveDown => app.move_cursor(1),
            Action::PageUp => app.move_cursor(-10),
            Action::PageDown => app.move_cursor(10),
            Action::Top => {
                if !app.sims.is_empty() {
                    app.table.select(Some(0));
                }
            }
            Action::Bottom => {
                if !app.sims.is_empty() {
                    app.table.select(Some(app.sims.len() - 1));
                }
            }
            Action::ToggleSelect => app.toggle_select(),
            Action::CycleSort => app.cycle_sort(),
            Action::Refresh => {
                if let Ok(sims) = simctl::list_devices() {
                    app = App::new(sims);
                    size_rx = start_size_scan(&app);
                }
            }
            Action::OpenConfirm => {
                if !app.selected.is_empty() {
                    app.modal = Modal::Confirm;
                }
            }
            Action::Confirm => {
                let targets: Vec<(Udid, bool)> = app
                    .selected_sims()
                    .iter()
                    .map(|s| (s.udid.clone(), s.state == simctl::DeviceState::Booted))
                    .collect();
                app.modal = Modal::Deleting;
                delete_rx = Some(spawn_delete(targets));
            }
            Action::CancelModal => app.modal = Modal::None,
            Action::DismissError => app.modal = Modal::None,
            Action::Nothing => {}
        }
    }
    Ok(())
}

fn start_size_scan(app: &App) -> Receiver<SizeUpdate> {
    let jobs: Vec<(Udid, String)> = app
        .sims
        .iter()
        .map(|s| (s.udid.clone(), s.data_path.clone()))
        .collect();
    sizing::spawn_size_scan(jobs)
}

fn spawn_delete(targets: Vec<(Udid, bool)>) -> Receiver<DeleteOutcome> {
    let (tx, rx) = std::sync::mpsc::channel();
    thread::spawn(move || {
        let mut booted_errors: Vec<String> = Vec::new();
        for (udid, booted) in &targets {
            if *booted
                && let Err(e) = simctl::shutdown(udid)
            {
                booted_errors.push(format!("{udid}: {e}"));
            }
        }

        let udids: Vec<Udid> = targets.iter().map(|(u, _)| u.clone()).collect();
        match simctl::delete(&udids) {
            Ok(()) => {
                let _ = tx.send(DeleteOutcome::Done(udids));
            }
            Err(e) => {
                let mut msg = format!("{e}");
                if !booted_errors.is_empty() {
                    msg.push_str("\n\nShutdown errors:\n");
                    msg.push_str(&booted_errors.join("\n"));
                }
                let _ = tx.send(DeleteOutcome::Failed(msg));
            }
        }
    });
    rx
}
