use configuration::{Configuration, INSTALL_DIR, CONFIG_FILE_NAME, RUN_BY_DEFAULT};
use std::env::{current_exe};
use std::fs::{copy, create_dir_all, remove_dir_all, remove_file, write, File};
use std::path::PathBuf;
use utils::{home_dir, current_exe_name};

pub fn install(config: &Configuration) -> Result<(), String> {
    let home_dir = home_dir()?;
    let startup_dir = get_startup_dir(&home_dir);
    let install_dir = home_dir.join(INSTALL_DIR);

    let mut config = (*config).clone();
    config.output_dir = install_dir.join("out")
        .to_string_lossy()
        .into_owned();

    info!("Copying executable to {:?}..", startup_dir);
    let current_executable = current_exe()
        .map_err(|e| format!("Could not find current executable: {}", e))?;
    let startup_executable = startup_dir.join(current_exe_name()?);
    copy(&current_executable, startup_executable)
        .map_err(|e| format!("Could not copy startup script: {}", e))?;

    create_dir_all(&install_dir).unwrap();

    info!("Creating configuration file..");
    let config_str = ::serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Could not serialize configuration: {}", e))?;
    let config_file = install_dir.join(CONFIG_FILE_NAME);
    write(&config_file, config_str)
        .map_err(|e| format!("Could not create configuration file: {}", e))?;

    info!("Finishing..");
    File::create(install_dir.join(RUN_BY_DEFAULT))
        .map_err(|error| format!("Could not create flag file: {}", error))?;

    Ok(())
}

pub fn uninstall() -> Result<(), String> {
    let home_dir = home_dir()?;
    let install_dir = home_dir.join(INSTALL_DIR);
    let executable = get_startup_dir(&home_dir).join("heaven-on-earth.exe");

    remove_file(executable)
        .map_err(|e| format!("Could not remove executable: {}", e))?;

    remove_dir_all(install_dir)
        .map_err(|e| format!("Could not remove install directory: {}", e))?;

    Ok(())
}

fn get_startup_dir(home: &PathBuf) -> PathBuf {
    home.join("AppData\\Roaming\\Microsoft\\Windows\\Start Menu\\Programs\\Startup")
}
