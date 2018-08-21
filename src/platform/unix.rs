pub fn set_wallpaper(path: &str) -> Result<(), ()> {
    use std::process::Command;

    let result = Command::new("feh").arg("--bg-fill").arg(path).status()?.success();
    match result {
        true => Ok(()),
        false => Err(())
    }
}