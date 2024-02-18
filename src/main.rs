#![warn(clippy::all, clippy::pedantic,clippy::restriction)]
#![allow(
    clippy::missing_docs_in_private_items,
    clippy::implicit_return,
    clippy::shadow_reuse,
    clippy::print_stdout,
    clippy::wildcard_enum_match_arm,
    clippy::else_if_without_else,
)]
mod editor;
mod row;
mod document;
mod terminal;
pub use terminal::Terminal;
use editor::Editor;
pub use document::Document;
pub use row::Row;
pub use editor::Position;
//Manual way to get control byte
// fn to_control_byte(c:char)->u8{
//     let byte = c as u8;
//     print!("{:?}",byte & 0b0001_1111);
//     byte & 0b0001_1111
// }

fn main() {
    Editor::default().run();
}
