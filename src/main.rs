// uncomment to avoid unnecessary console window on Windows
// #![windows_subsystem = "windows"]

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
use configuration::{Configuration, RESOURCES_DIR, RUN_BY_DEFAULT};
use platform::{install, uninstall};
use platform::set_wallpaper;
use schedule::{Agenda, Job};
use simplelog::{Config, LevelFilter, TermLogger};
use std::thread::sleep;
use std::time::Duration;
use wallpaper::Wallpaper;

mod configuration;
mod platform;
mod reddit;
mod utils;
mod wallpaper;

fn main() {
    TermLogger::init(LevelFilter::Info, Config::default()).unwrap();

    let yaml = load_yaml!("cli.yml");
    let mut app = App::from_yaml(yaml);
    let matches = app.clone().get_matches();

    fn load_config<F>(matches: Option<&ArgMatches>, after: F)
        where
            F: Fn(Configuration) -> (),
    {
        let matches = matches.map(ToOwned::to_owned).unwrap_or_default();
        match Configuration::init(&matches) {
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
        ("run", matches) => load_config(matches, |cfg| run(&cfg)),

        ("install", matches) => load_config(matches, |cfg| match install(&cfg) {
            Ok(()) => info!("Installation succeeded!"),
            Err(e) => error!("Installation failed: {}", e),
        }),

        ("uninstall", _) => match uninstall() {
            Ok(()) => info!("Uninstallation succeeded!"),
            Err(e) => error!("Uninstallation failed: {}", e),
        },

        (_, matches) => if ::utils::current_exe_dir()
            .join(RESOURCES_DIR)
            .join(RUN_BY_DEFAULT).is_file() {
            info!("file '{}' found", RUN_BY_DEFAULT);
            load_config(matches, |cfg| run(&cfg))
        } else {
            app.print_help().unwrap();
        },
    }
}

fn find_wallpaper(config: &Configuration) -> Option<Wallpaper> {
    // 'true' if the wallpaper matches the query set in the configuration, else 'false'
    fn wallpaper_ok(wall: &Wallpaper, cfg: &Configuration) -> bool {
        let ratio = match wall.ratio() {
            Some(ratio) => ratio,
            None => return false,
        };

        let wide_enough = cfg.min_ratio.map(|min| ratio >= min).unwrap_or(true);
        let tall_enough = cfg.max_ratio.map(|max| ratio <= max).unwrap_or(true);

        wide_enough && tall_enough
    }

    // the directory for saving downloaded images
    let out = &config.output_dir;

    for wallpaper in Wallpaper::search_on_reddit(config).iter_mut() {
        // download every wallpaper
        match wallpaper.download() {
            Ok(data) => if wallpaper_ok(wallpaper, config) {
                match wallpaper.save(out, &data) {
                    Ok(_) => return Some(wallpaper.clone()),
                    Err(e) => warn!("Downloaded wallpaper could not be saved: {}", e),
                }
            },
            Err(e) => warn!("Wallpaper could not be downloaded: {}", e),
        }
    }

    // we have not found a wallpaper
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
