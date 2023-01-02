pub use x11rb::connection::Connection;
pub use x11rb::properties::WmClass;
pub use x11rb::protocol::xproto::*;
pub use x11rb::protocol::Event;
pub use x11rb::xcb_ffi::XCBConnection;
pub use xkbcommon::xkb as xkbc;

use super::config::CONFIG;

use x11rb::atom_manager;
use x11rb::connection::RequestConnection as _;
use x11rb::errors::ReplyOrIdError;
use x11rb::protocol::xkb::{self, ConnectionExt as _};
use x11rb::protocol::xproto::{
    self, ConnectionExt as _, CreateWindowAux, EventMask, PropMode, WindowClass,
};
use x11rb::wrapper::ConnectionExt as _;

pub fn get_wm_name<Conn: Connection>(conn: &Conn, window: Window) -> anyhow::Result<String> {
    let reply = get_property(
        conn,
        false,
        window,
        AtomEnum::WM_CLASS,
        AtomEnum::STRING,
        0,
        2048,
    )?
    .reply()?;

    Ok(String::from_utf8(reply.value).expect("Something went wrong in WM_CLASS"))
}

pub fn get_inkscape_id<Conn: Connection>(conn: &Conn, screen: &Screen) -> Option<u32> {
    let tree = query_tree(conn, screen.root).unwrap();
    let windows = tree.reply().unwrap().children;

    for window in windows.iter() {
        if let Ok(name) = get_wm_name(conn, *window) {
            if name.contains("inkscape") || name.contains("Inkscape") {
                return Some(*window);
            }
        }
    }

    None
}

pub fn grab_keyboard<Conn: Connection>(conn: &Conn, window: Window) {
    grab_key(
        conn,
        false,
        window,
        ModMask::ANY,
        Grab::ANY,
        GrabMode::ASYNC,
        GrabMode::ASYNC,
    )
    .unwrap();
}

pub fn key_to_char(key: u8, state: &xkbc::State) -> String {
    let sym = state.key_get_one_sym(key.into());
    let utf8 = state.key_get_utf8(key.into());

    utf8
}

pub fn filter_key<Conn: Connection>(
    conn: &Conn,
    window: Window,
    event: Event,
    state: &xkbc::State,
) -> anyhow::Result<()> {
    let key = match event {
        Event::KeyPress(e) | Event::KeyRelease(e) => e.detail,
        _ => {
            return Ok(());
        }
    };

    let letter: &str = &key_to_char(key, state);

    println!("{}, {}", key, letter);

    let motions = &CONFIG.motions;

    if motions.contains(&letter) {
        if let Event::KeyPress(_) = event {
            CONFIG.execute(letter);
            // let act = motions(&letter).unwrap();
            // act();
        }
    } else {
        if let Event::KeyPress(e) = event {
            println!("I pressed {letter}");
            send_event(conn, false, window, EventMask::KEY_PRESS, e)?;
        }

        if let Event::KeyRelease(e) = event {
            println!("I released {letter}");
            send_event(conn, false, window, EventMask::KEY_RELEASE, e)?;
        }
    }

    Ok(())
}
