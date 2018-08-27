use configuration::Configuration;
use std::error::Error;
use std::process::Command;

pub fn set_wallpaper(path: &str) -> Result<(), ()> {
    let mut command = Command::new("feh");
    command.arg("--bg-fill").arg(path);

    match command.status() {
        Ok(status) => if status.success() {
            Ok(())
        } else {
            error!("Command exited with code {}", status);
            Err(())
        },
        Err(e) => {
            error!("Could not obtain exit code: {}", e);
            Err(())
        }
    }
}

pub fn install(config: &Configuration) -> Result<(), String> {
    Err("Your platform is not supported".to_owned())
}

pub fn uninstall() -> Result<(), String> {
    Err("Your platform is not supported".to_owned())
}
