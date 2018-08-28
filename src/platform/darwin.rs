use configuration::Configuration;
use std::error::Error;
use std::process::Command;

pub fn install(config: &Configuration) -> Result<(), String> {
    Err("Your platform is not supported".to_owned())
}

pub fn uninstall() -> Result<(), String> {
    Err("Your platform is not supported".to_owned())
}
