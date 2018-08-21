extern crate winapi;

use std::io::Error;
use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use self::winapi::um::winuser::{SystemParametersInfoW, SPIF_SENDCHANGE, SPIF_UPDATEINIFILE,
                                SPI_SETDESKWALLPAPER};
use self::winapi::shared::minwindef::TRUE;
use self::winapi::um::winnt::PVOID;

pub fn set_wallpaper(path: &str) -> Result<(), Error> {
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
        Err(Error::last_os_error())
    }
}