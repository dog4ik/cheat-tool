#![feature(try_blocks)]
#![feature(slice_as_chunks)]
#![feature(iter_array_chunks)]
#![feature(let_chains)]
#![feature(array_windows)]
pub mod api;
pub mod db;
pub mod emit_keypress;
pub mod process;
pub mod scripts;
pub mod watch_keypress;
pub use process::Process;
pub use process::ProcessError;
pub use watch_keypress::get_key_press;
