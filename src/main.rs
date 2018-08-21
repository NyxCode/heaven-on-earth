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
use clap::App;
use std::time::Duration;
use std::thread::sleep;
use simplelog::{TermLogger, LevelFilter, Config};
use std::path::Path;

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

fn main() {
    TermLogger::init(LevelFilter::Info, Config::default()).unwrap();

    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let mode = matches.value_of("mode").unwrap();
    let span = matches.value_of("span");
    let mode = reddit::Mode::from_identifier(mode, span).unwrap();
    let min_ratio = matches.value_of("min-ratio").map(|i| meval::eval_str(i).unwrap()).unwrap();
    let max_ratio = matches.value_of("max-ratio").map(|i| meval::eval_str(i).unwrap()).unwrap();
    let query_size = matches.value_of("query-size")
        .map(|i| meval::eval_str(i).unwrap()).unwrap() as u8;
    let run_every = matches.value_of("run-every");
    let output_dir = matches.value_of("output-dir").unwrap();

    info!("mode: {:?} | output-dir: {} | cron-expr: {:?} | ratio: {}-{} | query-size: {}",
          mode, output_dir, run_every, min_ratio, max_ratio, query_size);

    match run_every {
        None => run(&mode, output_dir, min_ratio, max_ratio, query_size),
        Some(expr) => run_repeating(&mode, output_dir, expr, min_ratio, max_ratio, query_size)
    };
}

fn run_repeating<P: AsRef<Path>>(query_mode: &reddit::Mode, output_dir: P, cron_expr: &str,
                                 min_ratio: f64, max_ratio: f64, query_size: u8) {
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
                       min_ratio: f64, max_ratio: f64, query_size: u8) {
    info!("Querying a new wallpaper...");
    match find_wallpaper(query_mode, output_dir, min_ratio as f32, max_ratio as f32, query_size) {
        Some(wallpaper) => wallpaper.set().unwrap(),
        None => warn!("No wallpaper found!")
    }
}