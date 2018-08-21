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

fn find_wallpaper(mode: &reddit::Mode, output_dir: &str, min_ratio: f32, max_ratio: f32) -> Wallpaper {
    let mut wallpapers = Wallpaper::search_on_reddit(mode, 30);
    for wallpaper in wallpapers.iter_mut() {
        if wallpaper.is_saved(output_dir) {
            info!("Wallpaper already downloaded, not downloading again..");
            return wallpaper.clone();
        }
        match wallpaper.download() {
            Ok(image_data) => {
                let ratio = wallpaper.ratio().unwrap();
                if ratio >= min_ratio && ratio <= max_ratio {
                    let save_res = wallpaper.save(output_dir, &image_data);
                    match save_res {
                        Ok(()) => return wallpaper.clone(),
                        Err(e) => warn!("Downloaded wallpaper could not be saved: {}", e)
                    }
                }
            }
            Err(e) => warn!("Wallpaper could not be downloaded: {}", e)
        }
    }
    panic!("No wallpaper found!")
}

fn main() {
    TermLogger::init(LevelFilter::Info, Config::default()).unwrap();

    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let mode = match &*matches.value_of("mode").unwrap().to_lowercase() {
        "new" => reddit::Mode::New,
        "hot" => reddit::Mode::Hot,
        "top" => {
            let span_id = matches.value_of("span").unwrap();
            let span = reddit::Span::from_identifier(span_id).unwrap();
            reddit::Mode::Top(span)
        }
        unsupported => {
            error!("Unsupported mode '{}'", unsupported);
            return;
        }
    };

    let min_ratio = matches.value_of("min-ratio")
        .map(|i| meval::eval_str(i).unwrap()).unwrap();
    let max_ratio = matches.value_of("max-ratio")
        .map(|i| meval::eval_str(i).unwrap()).unwrap();
    let run_every = matches.value_of("run-every");
    let out = matches.value_of("output-dir").unwrap();

    info!("mode: {:?} | output-dir: {} | cron-expr: {:?} | min-ratio: {} | max-ratio: {}",
          mode, out, run_every, min_ratio, max_ratio);


    match run_every {
        None => run(&mode, out, min_ratio, max_ratio),
        Some(expr) => run_repeating(&mode, out, expr, min_ratio, max_ratio)
    };
}

fn run_repeating(mode: &reddit::Mode, output: &str, expr: &str, min_ratio: f64, max_ratio: f64) {
    let mut agenda = Agenda::new();

    agenda.add(Job::new(|| {
        run(mode, output, min_ratio, max_ratio);
    }, expr.parse().unwrap()));

    loop {
        agenda.run_pending();
        sleep(Duration::from_secs(1));
    }
}

fn run(mode: &reddit::Mode, output: &str, min_ratio: f64, max_ratio: f64) {
    info!("Querying a new wallpaper...");
    let wallpaper = find_wallpaper(mode, output, min_ratio as f32, max_ratio as f32);
    wallpaper.set().unwrap();
}