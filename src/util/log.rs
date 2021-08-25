use lliw::{Fg, Reset, Style};

pub fn info(msg: &str) {
    println!(
        "{}{}{:>9} {}{}",
        Style::Bold,
        Fg::LightPurple,
        "Info",
        Reset,
        msg
    );
}

pub fn error(msg: &str) {
    println!("{}{}{:>9} {}{}", Style::Bold, Fg::Red, "Error", Reset, msg);
}

pub fn connected(msg: &str) {
    println!(
        "{}{}{:>9} {}{}",
        Style::Bold,
        Fg::LightBlue,
        "Connected",
        Reset,
        msg
    );
}

pub fn success(msg: &str) {
    println!(
        "{}{}{:>9} {}{}",
        Style::Bold,
        Fg::LightGreen,
        "Success",
        Reset,
        msg
    );
}

pub fn received(msg: &str) {
    println!(
        "{}{}{:>9} {}{}",
        Style::Bold,
        Fg::LightPurple,
        "Received",
        Reset,
        msg
    );
}
