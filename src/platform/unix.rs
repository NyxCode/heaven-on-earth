// credits: https://brandonio21.com/2017/06/setting-a-windows-desktop-background-in-rust/
pub fn set_wallpaper(path: &str) -> Result<(), ()> {
    use std::process::Command;

    let result = Command::new("feh").arg("--bg-fill").arg(path).status()?.success();
    match result {
        true => Ok(()),
        false => Err(())
    }
}