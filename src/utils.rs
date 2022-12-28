pub use x11rb::connection::Connection;
pub use x11rb::properties::WmClass;
pub use x11rb::protocol::xproto::*;
pub use x11rb::protocol::Event;

pub fn get_wm_name<Conn: Connection>(conn: &Conn, window: Window) -> String {
    let reply = get_property(
        conn,
        false,
        window,
        AtomEnum::WM_CLASS,
        AtomEnum::STRING,
        0,
        2048,
    )
    .unwrap()
    .reply()
    .unwrap();

    String::from_utf8(reply.value).unwrap()
}

pub fn get_inkscape_id<Conn: Connection>(conn: &Conn, screen: &Screen) -> Option<u32> {
    let tree = query_tree(conn, screen.root);
    let windows = tree.unwrap().reply().unwrap().children;

    for window in windows.iter() {
        let name = get_wm_name(conn, *window);

        if name.contains("inkscape") || name.contains("Inkscape") {
            return Some(*window);
        }
    }

    None
}

pub fn grab_keyboard<Conn: Connection>(conn: &Conn, window: Window) {
    grab_key(
        conn,
        false,
        window,
        ModMask::ANY,
        Grab::ANY,
        GrabMode::ASYNC,
        GrabMode::ASYNC,
    );
}

pub fn handle_event<Conn: Connection>(conn: &Conn, event: KeyPressEvent) {
    let key = event.detail;

    println!("{key}");
}
