/*
TODO: encapsulte all these ui components so we don't have to pass around
things like `curr_focus`, etc in the handler functions
*/

pub mod components;
pub mod modal;
pub mod notification;
pub mod pages;
pub mod event;

use pages::{Page, draw_page, handle_page_events};

use event::{Event, Key};
use crate::state::store::SharedAppStateStore;
use crate::GLOBAL_RUN;

use crossterm::{
    terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};

use tui::{
    backend::CrosstermBackend,
    Terminal,
};
use anyhow::Result;

use std::io::{self, Write};
use std::sync::atomic::Ordering;
use std::ops::DerefMut;

pub fn ui_loop(store: SharedAppStateStore) -> Result<()> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();

    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_len = store.lock().unwrap().state.app_config.tick_len;

    let evt_stream = event::async_event_reader(tick_len);

    let curr_page = Page::MainPage;

    terminal.clear()?;

    while GLOBAL_RUN.load(Ordering::Acquire) {
        // Update the stateful table from application state
        // Then release the lock
        let lock_store = store.lock().unwrap();
        // Clone for now, but maybe we shouldnt drop the store lock until the end of the loop?
        // TODO: use a borrow and drop the lock_store later?
        let notif = lock_store.state.notification.clone();
        let modal = lock_store.state.modal.clone();
        let selected = lock_store.state.get_selected_host();
        drop(lock_store);

        // Main draw loop
        terminal.draw(|f| {
            // FIXME: pass borrowed store instead of Arc
            draw_page(&curr_page, store.clone(), f);
            // TODO draw common elements controlled by appstate here,
            // ie Modal, notification, etc

            if let Some(notif) = notif {
                notification::draw_notification(notif, f)
            }

            if let Some(modal) = modal {
                // TODO: do this in a match instead of a branch
                if let Some(sel_host) = selected {
                    modal::draw_host_modal(modal, &sel_host, store.clone(), f)
                } else {
                    modal::draw_modal(modal, f)
                }
            }
        })?;

        // Global events handler
        // TODO: should these just return AppAction's and then we can
        // dispatch from here?
        //
        // TODO: profile deref-ing vs cloning the Arc here

        // Block until we either get an event or a timer tick
        // Changing to .iter() from .try_iter() (blocking vs non)
        if let Some(evt) = evt_stream.recv.iter().next() {
            match evt {
                Event::Key(key) => {
                    let mut lstore = store.lock().unwrap();

                    // TODO: use match
                    if lstore.state.modal.is_some() {
                        modal::handle_modal_event(key, lstore.deref_mut(), store.clone())
                    } else {
                        handle_page_events(&curr_page, key, lstore.deref_mut(), store.clone());
                    }

                    match key {
                        Key::Ctrl('c') | Key::Char('q') => GLOBAL_RUN.store(false, Ordering::Release),
                        _ => {}
                    }
                },

                Event::Tick => {
                    // Handle tick
                }
            }
        }
    }
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
