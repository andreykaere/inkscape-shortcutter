// use x11rb::atom_manager;
// use x11rb::connection::{Connection as _, RequestConnection as _};
// use x11rb::errors::ReplyOrIdError;
// use x11rb::protocol::xkb::{self, ConnectionExt as _};
// use x11rb::protocol::xproto::{
//     self, ConnectionExt as _, CreateWindowAux, EventMask, PropMode, WindowClass,
// };
// use x11rb::protocol::Event;
// use x11rb::wrapper::ConnectionExt as _;
use std::sync;
use std::thread;
use std::time::Duration;

mod utils;

use utils::*;

fn main() -> anyhow::Result<()> {
    let (conn, screen_num) = x11rb::connect(None)?;
    let mut inkscape_windows = get_inkscape_ids(&conn)?;

    if inkscape_windows.is_empty() {
        eprintln!("No Inkscape window was found, aborting ...");
        std::process::exit(2);
    };

    // let pool = rayon::ThreadPoolBuilder::new()
    //     .num_threads(20)
    //     .build()
    //     .unwrap();

    for inkscape_window in &inkscape_windows {
        println!("Found inkscape window, its id is {inkscape_window}");

        let win = *inkscape_window;
        thread::spawn(move || handle_inkscape_window(win).unwrap());
    }

    // Watch for new inkscape windows
    loop {
        let new_inkscape_windows = get_inkscape_ids(&conn)?;

        for window in &new_inkscape_windows {
            if !inkscape_windows.contains(window) {
                println!("New inkscape window, its id is {window}");

                let win = *window;
                thread::spawn(move || handle_inkscape_window(win).unwrap());

                inkscape_windows.push(*window);
            }
        }

        thread::sleep(Duration::from_millis(100));
    }

    Ok(())
}
