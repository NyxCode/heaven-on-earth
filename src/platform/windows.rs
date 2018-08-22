extern crate winapi;

use self::winapi::shared::minwindef::TRUE;
use self::winapi::um::winnt::PVOID;
use self::winapi::um::winuser::{
    SystemParametersInfoW, SPIF_SENDCHANGE, SPIF_UPDATEINIFILE, SPI_SETDESKWALLPAPER,
};
use reddit::Mode::*;
use std::env::{current_exe, home_dir};
use std::ffi::OsStr;
use std::fs::{copy, create_dir_all, remove_dir_all, remove_file, write};
use std::io::Error as IoError;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use Configuration;

const SCRIPT_NAME: &'static str = "heaven-on-earth.bat";

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
    let exe = current_exe().map_err(|e| format!("Could not find current executable: {}", e))?;
    let home = home_dir().ok_or_else(|| format!("Could not locate home directory"))?;
    let app = get_app_dir(&home);
    create_dir_all(&app).map_err(|e| format!("Could not create installation directory: {}", e))?;
    let exe_name = exe.file_name().unwrap();
    let new_exe = app.join(exe_name);
    copy(&exe, &new_exe).map_err(|e| format!("Could not copy executable: {}", e))?;

    info!("Creating script...");
    let script = app.join(SCRIPT_NAME);
    let command = create_startup_script(&config, &new_exe);
    write(&script, command).map_err(|e| format!("Could not create startup script: {}", e))?;

    info!("Copying script...");
    let startup_dir = get_startup_dir(&home);
    let script_file_name = script.file_name().unwrap();
    let startup_script = startup_dir.join(script_file_name);
    copy(&script, startup_script).map_err(|e| format!("Could not copy startup script: {}", e))?;

    Ok(())
}

pub fn uninstall() -> Result<(), String> {
    let home = home_dir().ok_or_else(|| format!("Could not locate home directory"))?;
    let app = get_app_dir(&home);
    let script = get_startup_dir(&home).join(SCRIPT_NAME);

    remove_dir_all(app).map_err(|e| format!("Could not remove app directory: {}", e))?;
    remove_file(script).map_err(|e| format!("Could not remove startup script: {}", e))?;

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

    if let Some(ref cron_expr) = config.run_every {
        cmd.push_str(" --run-every=");
        cmd.push_str(&cron_expr);
    }

    cmd
}

fn get_startup_dir(home: &PathBuf) -> PathBuf {
    home.join("AppData\\Roaming\\Microsoft\\Windows\\Start Menu\\Programs\\Startup")
}

fn get_app_dir(home: &PathBuf) -> PathBuf {
    home.join(".heaven-on-earth")
}
