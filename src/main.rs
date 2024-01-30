use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

mod utils;
use utils::{get_inkscape_ids, handle_inkscape_window};

fn main() -> anyhow::Result<()> {
    let (conn, _) = x11rb::connect(None)?;
    let cache = Arc::new(Mutex::new(HashSet::new()));

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(10)
        .build()
        .unwrap();

    loop {
        let all_ink_winds = get_inkscape_ids(&conn)?;
        let mut ink_winds = cache.lock().unwrap();

        for window in &all_ink_winds {
            if !ink_winds.contains(window) {
                println!("Detected inkscape window, its id is {window}");

                let win = *window;
                let cache_clone = Arc::clone(&cache);
                pool.spawn(move || handle_inkscape_window(win, cache_clone).unwrap());

                ink_winds.insert(*window);
            }
        }

        thread::sleep(Duration::from_millis(100));
    }
}
