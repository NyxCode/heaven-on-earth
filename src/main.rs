extern crate byteorder;
extern crate immeta;
extern crate reqwest;
extern crate schedule;
extern crate serde_json;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate meval;
extern crate rand;
extern crate simplelog;

mod configuration;
mod platform;
mod reddit;
mod wallpaper;

use clap::App;
use configuration::Configuration;
use platform::set_wallpaper;
use schedule::{Agenda, Job};
use simplelog::{Config, LevelFilter, TermLogger};
use std::thread::sleep;
use std::time::Duration;
use wallpaper::Wallpaper;

fn main() {
    TermLogger::init(LevelFilter::Info, Config::default()).unwrap();

    let yaml = load_yaml!("cli.yml");
    let mut app = App::from_yaml(yaml);
    let matches = app.clone().get_matches();

    if let Some(matches) = matches.subcommand_matches("run") {
        let config = Configuration::from_matches(&matches);
        run(&config)
    } else if let Some(matches) = matches.subcommand_matches("install") {
        let config = Configuration::from_matches(&matches);
        match platform::install(config) {
            Ok(()) => info!("Installation succeeded!"),
            Err(e) => error!("Installation failed: {}", e),
        }
    } else if let Some(_) = matches.subcommand_matches("uninstall") {
        match platform::uninstall() {
            Ok(()) => info!("Uninstallation succeeded!"),
            Err(e) => error!("Uninstallation failed: {}", e),
        }
    } else {
        app.print_help().unwrap();
    }
}

fn find_wallpaper(config: &Configuration) -> Option<Wallpaper> {
    let mut wallpapers = Wallpaper::search_on_reddit(config);

    fn wallpaper_ok(wall: &Wallpaper, config: &Configuration) -> bool {
        match wall.ratio() {
            Some(ratio) => ratio >= config.min_ratio && ratio <= config.max_ratio,
            _ => false
        }
    }

    for wallpaper in wallpapers.iter_mut() {
       /* if wallpaper_ok(wallpaper, config) {
            info!("Wallpaper already downloaded, not downloading it again..");
            return Some(wallpaper.clone());
        }*/

        match wallpaper.download() {
            Ok(image_data) => {
                if wallpaper_ok(wallpaper, config) {
                    match wallpaper.save(&config.output_dir, &image_data) {
                        Ok(()) => return Some(wallpaper.clone()),
                        Err(e) => warn!("Downloaded wallpaper could not be saved: {}", e),
                    }
                }
            }
            Err(e) => warn!("Wallpaper could not be downloaded: {}", e),
        }
    }
    None
}

fn run(config: &Configuration) {
    fn run_once(config: &Configuration) {
        info!("Searching for a new wallpaper...");
        match find_wallpaper(config) {
            Some(wallpaper) => match wallpaper.set() {
                Ok(_) => (),
                Err(err) => error!("Could not set wallpaper: {}", err),
            },
            None => warn!("No wallpaper found!"),
        };
    }

    fn run_repeating(config: &Configuration, cron_expr: &String) {
        let mut agenda = Agenda::new();
        agenda.add(Job::new(|| run_once(config), cron_expr.parse().unwrap()));

        loop {
            agenda.run_pending();
            sleep(Duration::from_secs(1));
        }
    }

    match config.run_every {
        Some(ref cron_expr) => run_repeating(config, cron_expr),
        None => run_once(config),
    }
}
