use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;
use x11rb::protocol::Event;
use x11rb::xcb_ffi::XCBConnection;

use x11rb::connection::RequestConnection as _;
use x11rb::protocol::xkb::{self, ConnectionExt as _};
use x11rb::protocol::xproto::EventMask;
use x11rb::wrapper::ConnectionExt as _;
use xkbcommon::xkb as xkbc;

use std::sync::{self, Mutex};
use std::thread;
use std::time::Duration;

fn is_window_valid<Conn: Connection>(conn: &Conn, window: Window) -> bool {
    match conn.get_window_attributes(window).unwrap().reply() {
        Ok(_) => true,
        _ => false,
    }
}

fn get_wm_name<Conn: Connection>(conn: &Conn, window: Window) -> anyhow::Result<String> {
    let property = if let Ok(property) = conn
        .get_property(false, window, AtomEnum::WM_NAME, AtomEnum::STRING, 0, 1024)?
        .reply()
    {
        property
    } else {
        return Ok(String::new());
    };

    Ok(String::from_utf8(property.value)?)
}

fn get_wm_instance_class<Conn: Connection>(
    conn: &Conn,
    window: Window,
) -> anyhow::Result<(String, String)> {
    let property = if let Ok(property) = conn
        .get_property(false, window, AtomEnum::WM_CLASS, AtomEnum::STRING, 0, 1024)?
        .reply()
    {
        property
    } else {
        return Ok((String::new(), String::new()));
    };

    // println!("{:?}", property.value);

    let mut iter = property.value.split(|x| *x == 0);
    let wm_instance_opt = iter.next();
    let wm_class_opt = iter.next();

    let wm_instance = if let Some(bytes) = wm_instance_opt {
        String::from_utf8(bytes.to_vec())?
    } else {
        String::new()
    };

    let wm_class = if let Some(bytes) = wm_class_opt {
        String::from_utf8(bytes.to_vec())?
    } else {
        String::new()
    };

    Ok((wm_instance, wm_class))
}

pub fn get_inkscape_ids<Conn: Connection>(conn: &Conn) -> anyhow::Result<Vec<Window>> {
    let screen = &conn.setup().roots[0];
    let all_windows = conn.query_tree(screen.root)?.reply()?.children;
    let mut inkscape_ids = Vec::new();

    for window in all_windows {
        let (wm_instance, wm_class) = get_wm_instance_class(conn, window)?;
        let wm_name = get_wm_name(conn, window)?;

        // Checking `wm_name` is needed to avoid detecting pop-up windows in
        // inkscape
        if (wm_instance.as_str(), wm_class.as_str()) == ("org.inkscape.Inkscape", "Inkscape")
            && wm_name != "org.inkscape.Inkscape"
        {
            inkscape_ids.push(window);
        }
    }

    Ok(inkscape_ids)
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
    // todo!();
    //

    Ok(())
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

pub fn handle_inkscape_window(
    inkscape_window: Window,
    inkscape_windows: sync::Arc<Mutex<Vec<Window>>>,
) -> anyhow::Result<()> {
    let (xcb_conn, screen_num) = xcb::Connection::connect(None)?;
    let screen_num = usize::try_from(screen_num)?;
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

    grab_keyboard(&conn, inkscape_window);
    conn.flush()?;

    while is_window_valid(&conn, inkscape_window) {
        // println!("foo {inkscape_window}");

        let event = if let Some(event) = conn.poll_for_event()? {
            event
        } else {
            thread::sleep(Duration::from_millis(100));
            continue;
        };

        match event {
            Event::KeyPress(_) | Event::KeyRelease(_) => {
                filter_key(&conn, inkscape_window, event, &state)?;
            }

            _ => {}
        }
    }

    // println!("bye {inkscape_window}");

    let mut inkscape_windows_lock = inkscape_windows.lock().unwrap();
    inkscape_windows_lock.retain(|&x| x != inkscape_window);

    Ok(())
}
