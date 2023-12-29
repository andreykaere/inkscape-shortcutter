// use x11rb::atom_manager;
// use x11rb::connection::{Connection as _, RequestConnection as _};
// use x11rb::errors::ReplyOrIdError;
// use x11rb::protocol::xkb::{self, ConnectionExt as _};
// use x11rb::protocol::xproto::{
//     self, ConnectionExt as _, CreateWindowAux, EventMask, PropMode, WindowClass,
// };
// use x11rb::protocol::Event;
// use x11rb::wrapper::ConnectionExt as _;

// use x11rb::xcb_ffi::XCBConnection;
use xkbcommon::xkb as xkbc;

mod utils;

use utils::*;

fn main() -> anyhow::Result<()> {
    let (xcb_conn, screen_num) = xcb::Connection::connect(None)?;
    let screen_num = usize::try_from(screen_num).unwrap();
    // Now get us an x11rb connection using the same underlying libxcb connection
    let conn = {
        let raw_conn = xcb_conn.get_raw_conn().cast();
        unsafe { XCBConnection::from_raw_xcb_connection(raw_conn, false) }
    }?;

    conn.prefetch_extension_information(xkb::X11_EXTENSION_NAME)?;
    let xkb = conn.xkb_use_extension(1, 0)?;
    let xkb = xkb.reply()?;
    assert!(xkb.supported);

    let context = xkbc::Context::new(xkbc::CONTEXT_NO_FLAGS);
    let device_id = xkbc::x11::get_core_keyboard_device_id(&xcb_conn);
    let keymap = xkbc::x11::keymap_new_from_device(
        &context,
        &xcb_conn,
        device_id,
        xkbc::KEYMAP_COMPILE_NO_FLAGS,
    );
    let state = xkbc::x11::state_new_from_device(&keymap, &xcb_conn, device_id);

    let screen = &conn.setup().roots[screen_num];

    if let Some(inkscape_window) = get_inkscape_id(&conn, screen) {
        println!("Found inkscape window, its id is {inkscape_window}");

        grab_keyboard(&conn, inkscape_window);
        conn.flush()?;

        loop {
            let event = conn.wait_for_event()?;

            match event {
                Event::KeyPress(_) | Event::KeyRelease(_) => {
                    filter_key(&conn, inkscape_window, event, &state)?;
                }

                _ => {}
            };
        }
    } else {
        println!("No Inkscape window was found, aborting ...");
        Ok(())
    }
}
