//#[cfg(not(debug_assertions))]
//#![windows_subsystem = "windows"]

#[macro_use]
extern crate clap;
extern crate immeta;
#[macro_use]
extern crate log;
extern crate meval;
extern crate rand;
extern crate reqwest;
extern crate schedule;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate simplelog;
extern crate toml;

use clap::{App, ArgMatches};
use configuration::*;
use platform::{install, uninstall};
use platform::set_wallpaper;
use schedule::{Agenda, Job};
use simplelog::{Config, LevelFilter, TermLogger};
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;
use wallpaper::Wallpaper;

mod configuration;
mod platform;
mod reddit;
mod wallpaper;
mod utils;

fn main() {
    TermLogger::init(LevelFilter::Info, Config::default()).unwrap();

    let yaml = load_yaml!("cli.yml");
    let mut app = App::from_yaml(yaml);
    let matches = app.clone().get_matches();

    fn load_config<F>(matches: Option<&ArgMatches>, after: F) where F: Fn(Configuration) -> () {
        let matches = match matches {
            Some(matches) => matches.to_owned(),
            None => ArgMatches::new()
        };
        let config = match configuration::init_config(&matches) {
            Ok(config) => {
                info!("{:?}", config);
                after(config)
            }
            Err(error) => {
                error!("{}", error);
            }
        };
    }

    match matches.subcommand() {
        ("run", matches) =>
            load_config(matches, |cfg| run(&cfg)),

        ("install", matches) =>
            load_config(matches, |cfg| match install(&cfg) {
                Ok(()) => info!("Installation succeeded!"),
                Err(e) => error!("Installation failed: {}", e),
            }),

        ("uninstall", _) =>
            match platform::uninstall() {
                Ok(()) => info!("Uninstallation succeeded!"),
                Err(e) => error!("Uninstallation failed: {}", e),
            }

        (x, matches) =>
            if configuration::should_run_by_default() {
                info!("file '{}' found", configuration::RUN_BY_DEFAULT);
                load_config(matches, |cfg| run(&cfg))
            } else {
                app.print_help();
            }
    }
}

fn find_wallpaper(config: &Configuration) -> Option<Wallpaper> {
    let mut wallpapers = Wallpaper::search_on_reddit(config);

    fn wallpaper_ok(wall: &Wallpaper, config: &Configuration) -> bool {
        let ratio = match wall.ratio() {
            Some(ratio) => ratio,
            None => return false
        };

        let wide_enough = config.min_ratio.map(|min| ratio >= min).unwrap_or(true);
        let tall_enough = config.max_ratio.map(|max| ratio <= max).unwrap_or(true);

        wide_enough && tall_enough
    }

    for wallpaper in wallpapers.iter_mut() {
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
