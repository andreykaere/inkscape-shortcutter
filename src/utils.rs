pub use x11rb::connection::Connection;
pub use x11rb::properties::WmClass;
pub use x11rb::protocol::xproto::*;
pub use x11rb::protocol::Event;
pub use x11rb::xcb_ffi::XCBConnection;

pub use x11rb::atom_manager;
pub use x11rb::connection::RequestConnection as _;
pub use x11rb::errors::ReplyOrIdError;
pub use x11rb::protocol::xkb::{self, ConnectionExt as _};
pub use x11rb::protocol::xproto::{
    self, ConnectionExt as _, CreateWindowAux, EventMask, PropMode, WindowClass,
};
pub use x11rb::wrapper::ConnectionExt as _;

pub use xkbcommon::xkb as xkbc;

pub fn get_wm_name<Conn: Connection>(
    conn: &Conn,
    window: Window,
) -> anyhow::Result<String> {
    let reply = conn
        .get_property(
            false,
            window,
            AtomEnum::WM_CLASS,
            AtomEnum::STRING,
            0,
            2048,
        )?
        .reply()?;

    Ok(String::from_utf8(reply.value)
        .expect("Something went wrong in WM_CLASS"))
}

pub fn get_inkscape_id<Conn: Connection>(
    conn: &Conn,
    screen: &Screen,
) -> Option<u32> {
    let tree = conn.query_tree(screen.root).unwrap();
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
    conn.grab_key(
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
    // let sym = state.key_get_one_sym(key.into());

    state.key_get_utf8(key.into())
}

pub fn execute_command(command: &str) -> anyhow::Result<()> {
    todo!();
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

    // let letter: &str = &key_to_char(key, state);
    let letter = key_to_char(key, state);

    println!("{}, {}", key, letter);

    if let Event::KeyPress(e) = event {
        if execute_command(&letter).is_err() {
            println!("I pressed {letter}");
            conn.send_event(true, window, EventMask::KEY_PRESS, e)?;
        }
    }

    if let Event::KeyRelease(e) = event {
        println!("I released {letter}");
        conn.send_event(true, window, EventMask::KEY_RELEASE, e)?;
    }

    conn.flush()?;
    conn.sync()?;

    Ok(())
}
