use clipboard::ClipboardContext;
use clipboard::ClipboardProvider;
use std::process::Command;

fn open_vim() {
    let mut vim = Command::new("alacritty")
        .args(["msg", "create-window"])
        // .args(["-name", "popup-bottom-center"])
        .args(["--class", "popup-bottom-center,scratchpad"])
        .args(["-e", "vim"])
        .args(["-u", "~/.minimal-tex-vimrc"])
        .spawn()
        .expect("Could not open vim");

    vim.wait().expect("Something went wrong while waiting");

    let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
    let text = ctx.get_contents().unwrap();
    println!("Text:\n{}", text);
}
