use clap::ArgMatches;
use meval::eval_str as str_to_i64;
use reddit::Mode;
use std::path::Path;

pub const CONFIG_FILE_NAME: &'static str = "config.json";
pub const RUN_BY_DEFAULT: &'static str = ".run-on-default";
pub const INSTALL_DIR: &'static str = ".heaven-on-earth";

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    pub mode: Option<Mode>,
    pub min_ratio: Option<f32>,
    pub max_ratio: Option<f32>,
    pub query_size: Option<u8>,
    pub run_every: Option<String>,
    pub output_dir: Option<String>,
    pub random: Option<bool>,
    pub subreddit: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Configuration {
    pub mode: Mode,
    pub min_ratio: Option<f32>,
    pub max_ratio: Option<f32>,
    pub query_size: u8,
    pub run_every: Option<String>,
    pub output_dir: String,
    pub random: bool,
    pub subreddit: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            mode: None,
            min_ratio: None,
            max_ratio: None,
            query_size: Some(50),
            run_every: None,
            output_dir: Some("image-out".to_string()),
            random: Some(false),
            subreddit: Some("EarthPorn".to_string()),
        }
    }
}

impl Settings {
    pub fn from_matches(matches: &ArgMatches) -> Result<Self, String> {
        let mode_str = matches.value_of("mode");
        let span = matches.value_of("span");
        let mode = mode_str.and_then(|string| {
            Mode::from_identifier(string, span)
                .map_err(|e| warn!("could not parse mode: {}", e))
                .ok()
        });
        let min_ratio = matches
            .value_of("min-ratio")
            .map(|i| str_to_i64(i).expect("could not parse min_ratio") as f32);
        let max_ratio = matches
            .value_of("max-ratio")
            .map(|i| str_to_i64(i).expect("could not parse max_ratio") as f32);
        let query_size = matches
            .value_of("query-size")
            .map(|i| str_to_i64(i).expect("could not parse query_size") as u8);
        let run_every = matches.value_of("run-every").map(|expr| expr.to_owned());
        let output_dir = matches.value_of("output-dir").map(|dir| dir.to_owned());
        let subreddit = matches.value_of("subreddit").map(|name| name.to_owned());
        let random: Option<bool> = matches
            .value_of("random")
            .map(|random| random.to_lowercase() == "true")
            .or_else(|| {
                if matches.is_present("random") {
                    Some(true)
                } else {
                    None
                }
            });

        let settings = Settings {
            mode,
            min_ratio,
            max_ratio,
            query_size,
            run_every,
            output_dir,
            random,
            subreddit,
        };

        Ok(settings)
    }

    pub fn combine(settings: Vec<Settings>) -> Result<Self, String> {
        fn get<T, F>(settings: &Vec<Settings>, selector: F) -> Option<T>
            where
                F: FnMut(&Settings) -> Option<T>,
        {
            settings.iter().filter_map(selector).last()
        }

        Ok(Settings {
            mode: get(&settings, |setting| setting.mode),
            min_ratio: get(&settings, |setting| setting.min_ratio),
            max_ratio: get(&settings, |setting| setting.max_ratio),
            query_size: get(&settings, |setting| setting.query_size),
            run_every: get(&settings, |setting| setting.run_every.clone()),
            output_dir: get(&settings, |setting| setting.output_dir.clone()),
            random: get(&settings, |setting| setting.random),
            subreddit: get(&settings, |setting| setting.subreddit.clone()),
        })
    }

    pub fn into_config(self) -> Result<Configuration, String> {
        fn get<T>(option: Option<T>, name: &str) -> Result<T, String> {
            option.ok_or_else(|| format!("Required setting '{}' missing", name))
        }

        Ok(Configuration {
            mode: get(self.mode, "mode")?,
            min_ratio: get(self.min_ratio, "min_ratio").ok(),
            max_ratio: get(self.max_ratio, "max_ratio").ok(),
            query_size: get(self.query_size, "query_size")?,
            run_every: self.run_every,
            output_dir: get(self.output_dir, "output_dir")?,
            random: get(self.random, "random")?,
            subreddit: get(self.subreddit, "subreddit")?,
        })
    }

    pub fn load_from_file<P: AsRef<Path>>(file: P) -> Result<Self, String> {
        let file: &Path = file.as_ref();
        let content = ::std::fs::read_to_string(file).unwrap();
        let config = ::serde_json::from_str(&content)
            .map_err(|error| format!("could not parse config: {}", error))?;
        Ok(config)
    }
}

impl Configuration {
    pub fn init(matches: &ArgMatches) -> Result<Configuration, String> {
        let file = ::utils::install_dir()?.join(CONFIG_FILE_NAME);

        let cli_settings = Settings::from_matches(matches)?;
        let default_settings = Settings::default();

        let settings = if file.is_file() {
            info!(
                "Loading configuration file {}...",
                file.file_name().unwrap().to_str().unwrap()
            );
            let file_config = Settings::load_from_file(file)?;
            vec![default_settings, file_config, cli_settings]
        } else {
            vec![default_settings, cli_settings]
        };

        let config = Settings::combine(settings)?.into_config()?;
        Ok(config)
    }
}