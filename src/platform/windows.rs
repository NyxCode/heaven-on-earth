extern crate winapi;

use self::winapi::shared::minwindef::TRUE;
use self::winapi::um::winnt::PVOID;
use self::winapi::um::winuser::{
    SystemParametersInfoW, SPIF_SENDCHANGE, SPIF_UPDATEINIFILE, SPI_SETDESKWALLPAPER,
};
use reddit::Mode::*;
use std::env::{current_exe, home_dir};
use std::error::Error;
use std::ffi::OsStr;
use std::fs::{copy, create_dir_all, remove_dir_all, remove_file, write};
use std::io::Error as IoError;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use Configuration;

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

pub fn install(config: Configuration) -> Result<(), String> {
    info!("Copying executable...");
    let executable = match current_exe() {
        Ok(exec) => exec,
        Err(e) => return Err(["could not locate executable: ", e.description()].concat()),
    };
    let home_dir = home_dir().unwrap();
    let app_dir = get_app_dir(&home_dir);
    if !app_dir.is_dir() {
        create_dir_all(&app_dir).unwrap();
    }
    let mut new_executable = app_dir.clone();
    new_executable.push(executable.file_name().unwrap());
    if new_executable.is_file() {
        return Err("Already installed!".to_owned());
    }
    copy(executable, &new_executable).unwrap();

    info!("Creating script...");
    let mut script_file = app_dir.clone();
    script_file.push("heaven-on-earth.bat");
    let command = create_startup_script(&config, &new_executable);
    write(&script_file, command).unwrap();

    info!("Copying script to autostart...");
    let startup_dir = get_autostart_dir(&home_dir);
    let startup_script = {
        let mut dir = startup_dir.clone();
        dir.push(script_file.file_name().unwrap());
        dir
    };
    copy(script_file, startup_script).unwrap();
    Ok(())
}

pub fn uninstall() -> Result<(), String> {
    let home_dir = home_dir().unwrap();
    let app_dir = get_app_dir(&home_dir);
    if app_dir.is_dir() {
        remove_dir_all(app_dir).unwrap();
    }

    let startup_dir = get_autostart_dir(&home_dir);
    let startup_script = {
        let mut dir = startup_dir.clone();
        dir.push("heaven-on-earth.bat");
        dir
    };
    if startup_script.is_file() {
        remove_file(startup_script).unwrap()
    }

    Ok(())
}

fn create_startup_script(config: &Configuration, executable: &PathBuf) -> String {
    let mut cmd = format!(
        "{} run -m={} --min-ratio={} --max-ratio={} --query-size={} --output-dir={}",
        executable.to_str().unwrap(),
        config.mode.identifier(),
        config.min_ratio,
        config.max_ratio,
        config.query_size,
        config.output_dir
    );
    match config.mode {
        Top(span) | Controversial(span) => {
            cmd.push_str(" --span=");
            cmd.push_str(span.identifier());
        }
        _ => (),
    };
    match config.run_every {
        Some(ref expr) => {
            cmd.push_str(" --run-every=");
            cmd.push_str(&expr);
        }
        _ => (),
    };
    cmd
}

fn get_autostart_dir(home: &PathBuf) -> PathBuf {
    let mut autostart = home.clone();
    autostart.push("AppData\\Roaming\\Microsoft\\Windows\\Start Menu\\Programs\\Startup");
    autostart
}

fn get_app_dir(home: &PathBuf) -> PathBuf {
    let mut app_dir = home.clone();
    app_dir.push(".heaven-on-earth");
    app_dir
}
