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
extern crate simplelog;
extern crate rand;

mod platform;
mod reddit;
mod wallpaper;

use clap::{App, ArgMatches};
use rand::{Rng, thread_rng};
use platform::set_wallpaper;
use schedule::{Agenda, Job};
use simplelog::{Config, LevelFilter, TermLogger};
use std::thread::sleep;
use std::time::Duration;
use wallpaper::Wallpaper;

#[derive(Debug)]
pub struct Configuration {
    pub mode: reddit::Mode,
    pub min_ratio: f32,
    pub max_ratio: f32,
    pub query_size: u8,
    pub run_every: Option<String>,
    pub output_dir: String,
    pub random: bool,
}

impl Configuration {
    fn from_matches(matches: &ArgMatches) -> Self {
        let mode = matches.value_of("mode").unwrap();
        let span = matches.value_of("span");
        let mode = reddit::Mode::from_identifier(mode, span).unwrap();
        let min_ratio = matches
            .value_of("min-ratio")
            .map(|i| meval::eval_str(i).unwrap())
            .unwrap() as f32;
        let max_ratio = matches
            .value_of("max-ratio")
            .map(|i| meval::eval_str(i).unwrap())
            .unwrap() as f32;
        let query_size = matches
            .value_of("query-size")
            .map(|i| meval::eval_str(i).unwrap())
            .unwrap() as u8;
        let run_every = matches.value_of("run-every").map(|expr| expr.to_owned());
        let output_dir = matches.value_of("output-dir").unwrap();
        let random = matches.is_present("random");

        let config = Configuration {
            mode,
            min_ratio,
            max_ratio,
            query_size,
            run_every,
            output_dir: output_dir.to_owned(),
            random,
        };

        info!("{:?}", config);
        config
    }
}

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
    let mut wallpapers = Wallpaper::search_on_reddit(&config.mode, config.query_size);

    if config.random {
        thread_rng().shuffle(&mut wallpapers);
    }

    for wallpaper in wallpapers.iter_mut() {
        wallpaper.update_state(&config.output_dir);

        if wallpaper.file.is_some() {
            info!("Wallpaper already downloaded, not downloading it again..");
            return Some(wallpaper.clone());
        }

        match wallpaper.download() {
            Ok(image_data) => {
                let ratio = wallpaper.ratio().unwrap();
                if ratio >= config.min_ratio && ratio <= config.max_ratio {
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
                Err(err) => error!("Could not set wallpaper: {}", err)
            }
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
        None => run_once(config)
    }
}