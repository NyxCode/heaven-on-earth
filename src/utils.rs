use std::path::PathBuf;

pub fn current_exe_name() -> Result<String, String> {
    ::std::env::current_exe()
        .map_err(|error| format!("Could not find current executable: {}", error))?
        .file_name()
        .ok_or_else(|| format!("Could not get filename"))?
        .to_str()
        .ok_or_else(|| format!("Could not convert to string"))
        .map(str::to_string)
}

pub fn home_dir() -> Result<PathBuf, String> {
    ::dirs::home_dir().ok_or_else(|| "Could not find home directory".to_string())
}

pub fn install_dir() -> PathBuf {
    let dir = home_dir().unwrap().join(::configuration::INSTALL_DIR);

    ::std::fs::create_dir_all(&dir).expect("Could not create install directory!");

    dir
}
