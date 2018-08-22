extern crate serde_json;
extern crate reqwest;
extern crate byteorder;
extern crate immeta;
extern crate schedule;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate simplelog;
extern crate meval;

mod wallpaper;
mod reddit;
mod platform;

use platform::set_wallpaper;
use wallpaper::Wallpaper;
use schedule::{Agenda, Job};
use clap::{App, ArgMatches};
use std::time::Duration;
use std::thread::sleep;
use simplelog::{TermLogger, LevelFilter, Config};
use std::path::Path;

#[derive(Debug)]
pub struct Configuration {
    pub mode: reddit::Mode,
    pub min_ratio: f32,
    pub max_ratio: f32,
    pub query_size: u8,
    pub run_every: Option<String>,
    pub output_dir: String,
}

impl Configuration {
    fn from_matches(matches: &ArgMatches) -> Self {
        let mode = matches.value_of("mode").unwrap();
        let span = matches.value_of("span");
        let mode = reddit::Mode::from_identifier(mode, span).unwrap();
        let min_ratio = matches.value_of("min-ratio")
            .map(|i| meval::eval_str(i).unwrap()).unwrap() as f32;
        let max_ratio = matches.value_of("max-ratio")
            .map(|i| meval::eval_str(i).unwrap()).unwrap() as f32;
        let query_size = matches.value_of("query-size")
            .map(|i| meval::eval_str(i).unwrap()).unwrap() as u8;
        let run_every = matches.value_of("run-every").map(|expr| expr.to_owned());
        let output_dir = matches.value_of("output-dir").unwrap();

        let config = Configuration {
            mode,
            min_ratio,
            max_ratio,
            query_size,
            run_every,
            output_dir: output_dir.to_owned(),
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
        match config.run_every {
            None => run(&config.mode, config.output_dir, config.min_ratio, config.max_ratio, config.query_size),
            Some(expr) => run_repeating(&config.mode, config.output_dir, expr.as_ref(), config.min_ratio, config.max_ratio, config.query_size)
        };
    } else if let Some(matches) = matches.subcommand_matches("install") {
        let config = Configuration::from_matches(&matches);
        match platform::install(config) {
            Ok(()) => info!("Installation succeeded!"),
            Err(e) => error!("Installation failed: {}", e)
        }
    } else if let Some(_) = matches.subcommand_matches("uninstall") {
        match platform::uninstall() {
            Ok(()) => info!("Uninstallation succeeded!"),
            Err(e) => error!("Uninstallation failed: {}", e)
        }
    } else {
        app.print_help().unwrap();
    }
}

fn find_wallpaper<P: AsRef<Path>>(query_mode: &reddit::Mode,
                                  output_dir: P,
                                  min_ratio: f32,
                                  max_ratio: f32,
                                  query_size: u8) -> Option<Wallpaper> {
    let mut wallpapers = Wallpaper::search_on_reddit(query_mode, query_size);
    for wallpaper in wallpapers.iter_mut() {
        wallpaper.update_state(&output_dir);

        if wallpaper.file.is_some() {
            info!("Wallpaper already downloaded, not downloading again..");
            return Some(wallpaper.clone());
        }

        match wallpaper.download() {
            Ok(image_data) => {
                let ratio = wallpaper.ratio().unwrap();
                if ratio >= min_ratio && ratio <= max_ratio {
                    match wallpaper.save(&output_dir, &image_data) {
                        Ok(()) => return Some(wallpaper.clone()),
                        Err(e) => warn!("Downloaded wallpaper could not be saved: {}", e)
                    }
                }
            }
            Err(e) => warn!("Wallpaper could not be downloaded: {}", e)
        }
    }
    None
}

fn run_repeating<P: AsRef<Path>>(query_mode: &reddit::Mode, output_dir: P, cron_expr: &str,
                                 min_ratio: f32, max_ratio: f32, query_size: u8) {
    let mut agenda = Agenda::new();

    let job = Job::new(|| run(query_mode, &output_dir, min_ratio, max_ratio, query_size),
                       cron_expr.parse().unwrap());
    agenda.add(job);

    loop {
        agenda.run_pending();
        sleep(Duration::from_secs(1));
    }
}

fn run<P: AsRef<Path>>(query_mode: &reddit::Mode, output_dir: P,
                       min_ratio: f32, max_ratio: f32, query_size: u8) {
    info!("Querying a new wallpaper...");
    match find_wallpaper(query_mode, output_dir, min_ratio, max_ratio, query_size) {
        Some(wallpaper) => wallpaper.set().unwrap(),
        None => warn!("No wallpaper found!")
    }
}