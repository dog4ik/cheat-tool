#![feature(try_blocks)]
#![feature(slice_as_chunks)]
#![feature(iter_array_chunks)]
#![feature(let_chains)]
#![feature(array_windows)]
pub mod process;
pub use process::Process;
pub use process::ProcessError;
