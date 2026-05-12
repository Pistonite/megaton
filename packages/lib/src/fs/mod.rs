pub mod fs;
mod fs_helpers;

#[allow(dead_code)]
pub fn init_stdio() {
    fs::initialize_fs();
}