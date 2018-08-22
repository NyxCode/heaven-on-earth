use Configuration;

pub fn set_wallpaper(path: &str) -> Result<(), ()> {
    use std::process::Command;

    let result = Command::new("feh").arg("--bg-fill").arg(path).status().unwrap().success();
    match result {
        true => Ok(()),
        false => Err(())
    }
}

pub fn install(config: Configuration) -> Result<(), String> {
    Err("Your platform is not supported".to_owned())
}

pub fn uninstall() -> Result<(), String> {
    Err("Your platform is not supported".to_owned())
}