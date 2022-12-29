mod config;
mod utils;

// use config::CONFIG;
use utils::*;

fn main() -> anyhow::Result<()> {
    let (conn, screen_num) = x11rb::connect(None)?;
    let screen = &conn.setup().roots[screen_num];

    if let Some(inkscape_window) = get_inkscape_id(&conn, screen) {
        println!("Found inkscape window, its id is {inkscape_window}");

        grab_keyboard(&conn, inkscape_window);
        conn.flush()?;

        loop {
            let event = conn.wait_for_event()?;

            match event {
                Event::KeyPress(_) | Event::KeyRelease(_) => {
                    filter_key(&conn, inkscape_window, event)?;
                }

                _ => {}
            };
        }
    } else {
        println!("No Inkscape window was found, aborting ...");
        Ok(())
    }
}
