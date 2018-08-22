#[cfg(windows)]
mod windows;
#[cfg(not(windows))]
mod unix;

#[cfg(windows)]
pub use self::windows::set_wallpaper;
#[cfg(not(windows))]
pub use self::unix::set_wallpaper;

#[cfg(windows)]
pub use self::windows::install;
#[cfg(not(windows))]
pub use self::unix::install;

#[cfg(windows)]
pub use self::windows::uninstall;
#[cfg(not(windows))]
pub use self::unix::uninstall;



