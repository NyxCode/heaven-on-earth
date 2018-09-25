#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[macro_use]
extern crate clap;
extern crate immeta;
#[macro_use]
extern crate log;
extern crate meval;
extern crate rand;
extern crate reqwest;
#[macro_use]
extern crate serde_derive;
extern crate dirs;
extern crate job_scheduler;
extern crate serde_json;
extern crate simplelog;
extern crate wallpaper as wallpaper_lib;

use clap::{App, ArgMatches};
use configuration::{Configuration, RUN_BY_DEFAULT};
use job_scheduler::{Job, JobScheduler};
use platform::{install, uninstall};
use simplelog::{CombinedLogger, Config, LevelFilter, TermLogger, WriteLogger};
use std::fs::File;
use std::thread::sleep;
use std::time::Duration;
use wallpaper::Wallpaper;

mod configuration;
mod platform;
mod reddit;
mod utils;
mod wallpaper;

fn main() {
    let log_file = utils::install_dir().join("latest.log");

    CombinedLogger::init(vec![
        #[cfg(debug_assertions)]
        TermLogger::new(LevelFilter::Info, Config::default()).unwrap(),
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            File::create(log_file).unwrap(),
        ),
    ]).unwrap();

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

        (_, matches) => if ::utils::install_dir().join(RUN_BY_DEFAULT).is_file() {
            info!("file '{}' found", RUN_BY_DEFAULT);
            load_config(matches, |cfg| run(&cfg))
        } else {
            app.print_help().unwrap();
        },
    }
}

fn run(config: &Configuration) {
    fn run_once(config: &Configuration) {
        info!("Searching for a new wallpaper...");
        match Wallpaper::find(config) {
            Some(wallpaper) => match wallpaper.set() {
                Ok(_) => (),
                Err(err) => error!("Could not set wallpaper: {}", err),
            },
            None => error!("No wallpaper found!"),
        };
    }

    fn run_repeating(config: &Configuration, cron_expr: &String) {
        run_once(config);

        let job = Job::new(cron_expr.parse().unwrap(), || run_once(config));

        let mut scheduler = JobScheduler::new();
        scheduler.add(job);

        loop {
            scheduler.tick();
            sleep(Duration::from_millis(500));
        }
    }

    match config.run_every {
        Some(ref cron_expr) => run_repeating(config, cron_expr),
        None => run_once(config),
    }
}
