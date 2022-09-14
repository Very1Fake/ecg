use std::env;

////////////////////////////////////////////////////////////////////////////////////////////////////
// Constants
////////////////////////////////////////////////////////////////////////////////////////////////////

pub static VERSION: &str = env!("CARGO_PKG_VERSION");

////////////////////////////////////////////////////////////////////////////////////////////////////
// Enums
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug)]
pub enum ExitCode {
    Ok = 0,
    OutOfMemory,
    OutOfVideoMemory,
}

impl ExitCode {
    pub fn as_int(&self) -> i32 {
        *self as i32
    }
}