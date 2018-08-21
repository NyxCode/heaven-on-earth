/*extern crate user32;

// credits: https://brandonio21.com/2017/06/setting-a-windows-desktop-background-in-rust/
pub fn set_wallpaper(path: &str) -> Result<(), ()> {
    use std::ffi::CString;
    use std::os::raw::c_void;

    // do work to get the pointer of our owned string
    let path_ptr = CString::new(path).unwrap();
    let path_ptr_c = path_ptr.into_raw();
    let result = unsafe {
        match path_ptr_c.is_null() {
            false => user32::SystemParametersInfoA(20, 0, path_ptr_c as *mut c_void, 0),
            true => 0
        }
    };
    // rust documentation says we must return the pointer this way
    unsafe {
        CString::from_raw(path_ptr_c)
    };
    match result {
        0 => Err(()),
        _ => Ok(())
    }
} */

use std::io::Error;
use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;

extern crate winapi;
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