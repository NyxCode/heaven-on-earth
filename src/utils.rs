use std::path::PathBuf;

pub fn current_exe_dir() -> PathBuf {
    ::std::env::current_exe()
        .expect("Could not find current executable")
        .parent()
        .expect("Could not find directory of current executable")
        .to_path_buf()
}