use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

mod utils;

use utils::*;

fn main() -> anyhow::Result<()> {
    let (conn, screen_num) = x11rb::connect(None)?;
    let mut inkscape_windows = Arc::new(Mutex::new(vec![]));

    // let pool = rayon::ThreadPoolBuilder::new()
    //     .num_threads(20)
    //     .build()
    //     .unwrap();

    // if inkscape_windows.is_empty() {
    //     println!("No Inkscape window was found, waiting for new windows ...");
    // } else {
    //     for inkscape_window in &inkscape_windows {
    //         println!("Found inkscape window, its id is {inkscape_window}");

    //         let win = *inkscape_window;
    //         thread::spawn(move || handle_inkscape_window(win, inkscape_windows).unwrap());
    //     }
    // }

    // Watch for new inkscape windows
    loop {
        // println!("fooooooo");

        let new_inkscape_windows = get_inkscape_ids(&conn)?;
        let mut inkscape_windows_lock = inkscape_windows.lock().unwrap();

        for window in &new_inkscape_windows {
            if !inkscape_windows_lock.contains(window) {
                println!("New inkscape window, its id is {window}");

                let win = *window;
                thread::spawn(move || handle_inkscape_window(win, inkscape_windows).unwrap());

                inkscape_windows_lock.push(*window);
            }
        }

        thread::sleep(Duration::from_millis(100));
    }
}
