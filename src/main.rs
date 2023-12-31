use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

mod utils;
use utils::{get_inkscape_ids, handle_inkscape_window};

fn main() -> anyhow::Result<()> {
    let (conn, _) = x11rb::connect(None)?;
    let inkscape_windows = Arc::new(Mutex::new(vec![]));

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(10)
        .build()
        .unwrap();

    loop {
        let new_inkscape_windows = get_inkscape_ids(&conn)?;
        let mut inkscape_windows_lock = inkscape_windows.lock().unwrap();

        for window in &new_inkscape_windows {
            if !inkscape_windows_lock.contains(window) {
                println!("Detected inkscape window, its id is {window}");

                let win = *window;
                let ink_windows = Arc::clone(&inkscape_windows);
                pool.spawn(move || handle_inkscape_window(win, ink_windows).unwrap());

                inkscape_windows_lock.push(*window);
            }
        }

        thread::sleep(Duration::from_millis(100));
    }
}
