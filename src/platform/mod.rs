#[cfg(target_os = "macos")]
pub use self::darwin::install;
#[cfg(target_os = "macos")]
pub use self::darwin::set_wallpaper;
#[cfg(target_os = "macos")]
pub use self::darwin::uninstall;
#[cfg(target_os = "linux")]
pub use self::linux::install;
#[cfg(target_os = "linux")]
pub use self::linux::set_wallpaper;
#[cfg(target_os = "linux")]
pub use self::linux::uninstall;
#[cfg(target_os = "windows")]
pub use self::windows::install;
#[cfg(target_os = "windows")]
pub use self::windows::set_wallpaper;
#[cfg(target_os = "windows")]
pub use self::windows::uninstall;

#[cfg(target_os = "macos")]
mod darwin;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;
