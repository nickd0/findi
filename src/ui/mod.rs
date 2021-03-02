/*
TODO: encapsulte all these ui components so we don't have to pass around
things like `curr_focus`, etc in the handler functions
*/

pub mod components;
pub mod modal;
pub mod notification;
pub mod pages;

use pages::{Page, draw_page, handle_page_events};

use crate::state::store::SharedAppStateStore;
use crate::GLOBAL_RUN;

use termion;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::event::{Key};
use tui::{
    backend::TermionBackend,
    Terminal,
};
use anyhow::Result;

use std::io;
use std::sync::atomic::Ordering;
use std::ops::DerefMut;

pub fn ui_loop(store: SharedAppStateStore) -> Result<()> {
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let lock_store = store.lock().unwrap();
    drop(lock_store);

    // Uses termions async stdin for now,
    // Does not work on windows
    let mut stdin = termion::async_stdin().keys();

    let curr_page = Page::MainPage;

    terminal.clear()?;
    // TODO control this from a separate thread using an Atomic::Bool

    while GLOBAL_RUN.load(Ordering::Acquire) {
        // Update the stateful table from application state
        // Then release the lock
        let lock_store = store.lock().unwrap();
        // Clone for now, but maybe we shouldnt drop the store lock until the end of the loop?
        // TODO: use a borrow and drop the lock_store later?
        let notif = lock_store.state.notification.clone();
        let modal = lock_store.state.modal.clone();
        drop(lock_store);

        // Main draw loop
        terminal.draw(|f| {
            draw_page(&curr_page, store.clone(), f);
            // TODO draw common elements controlled by appstate here,
            // ie Modal, notification, etc

            if let Some(notif) = notif {
                notification::draw_notification(notif, f)
            }

            if let Some(modal) = modal {
                modal::draw_modal(modal, f)
            }
        })?;

        // Global events handler
        // TODO: should these just return AppAction's and then we can
        // dispatch from here?
        //
        // TODO: profile deref-ing vs cloning the Arc here
        // 
        if let Some(Ok(key)) = stdin.next() {
            let mut lstore = store.lock().unwrap();
            if lstore.state.modal.is_some() {
                modal::handle_modal_event(key, lstore.deref_mut(), store.clone())
            }

            handle_page_events(&curr_page, key, lstore.deref_mut(), store.clone());

            match key {
                Key::Ctrl('c') | Key::Esc => GLOBAL_RUN.store(false, Ordering::Release),
                _ => {}
            }
        }
    }
    terminal.clear()?;

    Ok(())
}
