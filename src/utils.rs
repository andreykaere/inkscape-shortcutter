use x11rb::connection::Connection;
// use x11rb::protocol::xproto::*;
use x11rb::protocol::Event;
use x11rb::xcb_ffi::XCBConnection;

use x11rb::connection::RequestConnection as _;
use x11rb::protocol::xkb::{self, ConnectionExt as _, PerClientFlag};
use x11rb::protocol::xproto::{
    AtomEnum, ConnectionExt as _, EventMask, Grab, GrabMode, ModMask, Window,
};
use x11rb::wrapper::ConnectionExt as _;
use xkbcommon::xkb as xkbc;

use std::collections::HashSet;
use std::process::Command;
use std::sync::{self, Mutex};
use std::thread;
use std::time::Duration;

use anyhow::bail;
use mktemp::Temp;

#[derive(Default)]
pub struct Mods {
    shift: bool,
    alt: bool,
    ctrl: bool,
    super_key: bool,
}

pub struct Binding {
    letter: char,
    mods: Mods,
}

impl Binding {
    fn from_keys(input: &[String]) -> anyhow::Result<Self> {
        let mut mods = Mods::default();
        let mut keys = input.to_vec();

        if keys.contains(&"Shift".to_string()) {
            keys.retain(|x| x.as_str() != "Shift");
            mods.shift = true;
        }

        if keys.contains(&"Alt".to_string()) {
            keys.retain(|x| x.as_str() != "Alt");
            mods.alt = true;
        }

        if keys.contains(&"Ctrl".to_string()) {
            keys.retain(|x| x.as_str() != "Ctrl");
            mods.ctrl = true;
        }

        if keys.contains(&"Super".to_string()) {
            keys.retain(|x| x.as_str() != "Super");
            mods.super_key = true;
        }

        if keys.len() > 1 {
            bail!("Incorrect input");
        } else {
            let letter = keys[0].chars().next().unwrap();
            Ok(Self { letter, mods })
        }
    }
}

fn is_window_valid<Conn: Connection>(conn: &Conn, window: Window) -> bool {
    conn.get_window_attributes(window).unwrap().reply().is_ok()
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

    Ok(String::from_utf8_lossy(&property.value).to_string())
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
        String::from_utf8_lossy(bytes).to_string()
    } else {
        String::new()
    };

    let wm_class = if let Some(bytes) = wm_class_opt {
        String::from_utf8_lossy(bytes).to_string()
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

// pub fn grab_keyboard<Conn: Connection>(conn: &Conn, window: Window) {
//     conn.grab_key(
//         false,
//         window,
//         ModMask::ANY,
//         Grab::ANY,
//         GrabMode::ASYNC,
//         GrabMode::ASYNC,
//     )
//     .unwrap();
// }

// pub fn key_to_char(key: u8, state: &xkbc::State) -> String {
//     let sym = state.key_get_consumed_mods(key.into());

//     println!("sym: {:?}", sym);

//     state.key_get_utf8(key.into())
// }

pub fn parse_combination(string: &str) -> HashSet<String> {
    let mut comb = HashSet::new();

    for letter in string.split('+') {
        comb.insert(letter.to_string());
    }

    comb
    // HashSet::from(string.split('+').map(|x| x.to_string()).collect::Vec<_>().into())
}

pub fn open_editor() -> anyhow::Result<()> {
    Command::new("alacritty")
        .args([
            "msg",
            "create-window",
            "--class",
            "popup-bottom-center,scratchpad",
            "-e",
            "vim",
            "-u",
            "~/.minimal-tex-vimrc",
            // filename,
        ])
        .spawn()?;

    Ok(())

    // Command::new("urxvt")
    //     .args([
    //         "-name",
    //         "popup-bottom-center",
    //         "-e",
    //         "vim",
    //         "-u",
    //         "~/.minimal-tex-vimrc",
    //         filename,
    //     ])
    //     .spawn();
    // subprocess.run([
    //     'urxvt',
    //     '-name', 'popup-bottom-center',
    //     '-e', "vim",
    //     "-u", "~/.minimal-tex-vimrc",
    //     f"{filename}",
    // ])
}

pub fn execute_command(combination: &HashSet<String>) -> anyhow::Result<()> {
    if combination == &parse_combination("t") {
        println!("hello!");
        // let file = Temp::new_file()?.to_path_buf();
        // let filename = file.to_str().unwrap();
        // open_editor(&filename)?;
        open_editor()?;
    }
    if combination == &parse_combination("a+s") {
        todo!();
    }
    // todo!();
    //

    Ok(())
}

pub fn filter_key<Conn: Connection>(
    conn: &Conn,
    window: Window,
    raw_event: Event,
    state: &xkbc::State,
    device_id: u8,
    buffer: &mut HashSet<String>,
) -> anyhow::Result<()> {
    let event = match raw_event {
        Event::KeyPress(e) | Event::KeyRelease(e) => e,
        _ => {
            return Ok(());
        }
    };

    let key = event.detail;
    // let letter: &str = &key_to_char(key, state);
    // let letter = key_to_char(key, state);
    let letter = state.key_get_utf8(key.into());

    if let Event::KeyPress(event) = raw_event {
        println!("state: {:?}", event.state);
        // let foo: u32 = xproto::ModMask::SHIFT.into();
        // println!("{:?}", foo != 0);
        // println!("{:?}", xproto::ModMask::CONTROL as u32 != 0);
        // if execute_command(&letter).is_err() {
        println!("I pressed {letter}");

        // conn.send_event(false, window, EventMask::NO_EVENT, e)?;

        buffer.insert(letter.clone());
    }

    if let Event::KeyRelease(event) = raw_event {
        println!("I released {letter}");
        // conn.send_event(false, window, EventMask::NO_EVENT, e)?;

        // buffer.remove(&letter);
        println!("Buffer is: {:?}", buffer);
        execute_command(buffer)?;

        buffer.clear();
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

    conn.xkb_per_client_flags(
        device_id as u16,
        PerClientFlag::DETECTABLE_AUTO_REPEAT,
        PerClientFlag::DETECTABLE_AUTO_REPEAT,
        Default::default(),
        Default::default(),
        Default::default(),
    )?;

    conn.grab_key(
        false,
        inkscape_window,
        ModMask::ANY,
        Grab::ANY,
        GrabMode::ASYNC,
        GrabMode::ASYNC,
    )?;
    conn.flush()?;

    // Buffer for pressed keys. Basically this is the combination of the keys,
    // untill the release happens
    let mut buffer = HashSet::new();

    while is_window_valid(&conn, inkscape_window) {
        // println!("foo {inkscape_window}");

        let event = if let Some(event) = conn.poll_for_event()? {
            event
        } else {
            thread::sleep(Duration::from_millis(10));
            continue;
        };

        match event {
            Event::KeyPress(_) | Event::KeyRelease(_) => {
                filter_key(
                    &conn,
                    inkscape_window,
                    event,
                    &state,
                    device_id as u8,
                    &mut buffer,
                )?;
            }

            _ => {}
        }
    }

    // println!("bye {inkscape_window}");

    let mut inkscape_windows_lock = inkscape_windows.lock().unwrap();
    inkscape_windows_lock.retain(|&x| x != inkscape_window);

    Ok(())
}
