extern crate winapi;

use configuration::Configuration;
use self::winapi::shared::minwindef::TRUE;
use self::winapi::um::winnt::PVOID;
use self::winapi::um::winuser::{
    SPI_SETDESKWALLPAPER, SPIF_SENDCHANGE, SPIF_UPDATEINIFILE, SystemParametersInfoW,
};
use std::env::{current_exe, home_dir};
use std::ffi::OsStr;
use std::fs::{copy, create_dir_all, remove_dir_all, remove_file, write};
use std::io::Error as IoError;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;

pub fn set_wallpaper(path: &str) -> Result<(), ()> {
    let full_path: Vec<u16> = OsStr::new(path).encode_wide().chain(once(0)).collect();
    let ret = unsafe {
        SystemParametersInfoW(
            SPI_SETDESKWALLPAPER,
            0,
            full_path.as_ptr() as PVOID,
            SPIF_SENDCHANGE | SPIF_UPDATEINIFILE,
        )
    };
    if ret == TRUE {
        Ok(())
    } else {
        let err = IoError::last_os_error();
        error!("setting wallpaper failed: {}", err);
        Err(())
    }
}

pub fn install(config: &Configuration) -> Result<(), String> {
    let home_dir = home_dir()
        .ok_or_else(|| "Home directory unknown".to_string())?;
    let startup_dir = get_startup_dir(&home_dir);
    let mut config = (*config).clone();
    config.output_dir = startup_dir
        .join("heaven-on-earth")
        .join("out")
        .to_string_lossy().into_owned();

    info!("Copying executable to {:?}..", startup_dir);
    let current_executable = current_exe()
        .map_err(|e| format!("Could not find current executable: {}", e))?;
    let executable_name = current_executable.file_name().unwrap().to_str().unwrap();
    let startup_executable = startup_dir.join(executable_name);
    copy(&current_executable, startup_executable)
        .map_err(|e| format!("Could not copy startup script: {}", e))?;

    let resources_dir = startup_dir.join("heaven-on-earth");
    create_dir_all(&resources_dir).unwrap();

    info!("Creating configuration file..");
    let config_str = ::toml::to_string_pretty(&config)
        .map_err(|e| format!("Could not serialize configuration: {}", e))?;
    let config_file = resources_dir.join(::configuration::CONFIG_FILE_NAME);
    write(&config_file, config_str)
        .map_err(|e| format!("Could not create configuration file: {}", e))?;

    info!("Finishing..");
    let flag_file = resources_dir.join(::configuration::RUN_BY_DEFAULT);
    ::std::fs::File::create(flag_file).unwrap();
    Ok(())
}

pub fn uninstall() -> Result<(), String> {
    let home_dir = home_dir().ok_or_else(|| format!("Could not locate home directory"))?;
    let startup_dir = get_startup_dir(&home_dir);
    let executable = startup_dir.join("heaven-on-earth.exe");
    let config = startup_dir.join(::configuration::CONFIG_FILE_NAME);

    remove_file(executable)
        .map_err(|e| format!("Could not remove executable: {}", e))?;
    remove_dir_all(startup_dir.join("heaven-on-earth"))
        .map_err(|e| format!("Could not remove resources directory: {}", e))?;

    Ok(())
}

fn get_startup_dir(home: &PathBuf) -> PathBuf {
    home.join("AppData\\Roaming\\Microsoft\\Windows\\Start Menu\\Programs\\Startup")
}
