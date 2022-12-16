use lazy_static::lazy_static;

pub const ASYNC_THREADS: usize = 2;
pub const MIN_WINDOW_WIDTH: u32 = 854;
pub const MIN_WINDOW_HEIGHT: u32 = 480;

lazy_static! {
    pub static ref CPU_CORES: usize = num_cpus::get();
    pub static ref BLOCKING_THREADS: usize = (*CPU_CORES / 2).max(2);
}
