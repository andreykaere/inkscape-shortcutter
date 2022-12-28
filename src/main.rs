mod utils;

use utils::*;

lazy_static::lazy_static! {
    static ref CONFIG: Config = Config::new();
}

struct Motion {
    key: char,
    motion: fn(),
}

impl Motion {
    fn new(key: char, motion: fn()) -> Self {
        Self { key, motion }
    }
}

struct Config {
    motions: Vec<Motion>,
}

impl Config {
    fn new() -> Self {
        Self {
            motions: vec![Motion::new('s', solid)],
        }
    }

    fn get_keys(&self) -> Vec<char> {
        let mut keys = vec![];

        for motion in self.motions.iter() {
            keys.push(motion.key);
        }

        keys
    }
}

fn solid() {}

fn main() {
    let (conn, screen_num) = x11rb::connect(None).unwrap();
    let screen = &conn.setup().roots[screen_num];

    if let Some(id) = get_inkscape_id(&conn, screen) {
        println!("Found inkscape window, its id is {id}");

        grab_keyboard(&conn, id);
        conn.flush();

        loop {
            let event = conn.wait_for_event().unwrap();

            match event {
                Event::KeyPress(event) => {
                    println!("Key is pressed!");

                    handle_event(&conn, event);
                }

                Event::KeyRelease(event) => {
                    println!("Key is released!");

                    handle_event(&conn, event);
                }

                _ => {}
            };
        }
    } else {
        println!("No Inkscape window was found, aborting ...");
    }
}
