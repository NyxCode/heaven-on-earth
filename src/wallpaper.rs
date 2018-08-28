use configuration::Configuration;
use rand::{Rng, thread_rng};
use std::fs::{canonicalize, create_dir_all, File};
use std::fs::read_dir;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use super::immeta::{GenericMetadata::*, load_from_buf};
use super::reddit;
use super::reqwest;
use super::serde_json;
use super::serde_json::Value as JsonVal;

#[derive(Debug, Clone)]
pub struct Wallpaper {
    pub title: String,
    pub url: String,
    pub format: Option<String>,
    /// the file on which this wallpaper is stored or [None] if it hasn't been saved yet
    pub file: Option<PathBuf>,
    /// the dimensions (x, y) of this wallpaper or [None] if it hasn't been downloaded yet
    pub dimensions: Option<(u32, u32)>,
}

impl Wallpaper {
    /// Tries to find a single wallpaper on Reddit
    pub fn find(config: &Configuration) -> Option<Self> {
        // 'true' if the wallpaper matches the query set in the configuration, else 'false'
        fn wallpaper_ok(wall: &Wallpaper, cfg: &Configuration) -> bool {
            let ratio = match wall.ratio() {
                Some(ratio) => ratio,
                None => return false,
            };
            let size = match wall.megapixel() {
                Some(size) => size,
                None => return false,
            };

            let wide_enough = cfg.min_ratio.map(|min| ratio >= min).unwrap_or(true);
            let tall_enough = cfg.max_ratio.map(|max| ratio <= max).unwrap_or(true);
            let is_current = match ::wallpaper_lib::get() {
                Ok(path) => path.contains(&wall.construct_filename()),
                Err(_) => true
            };
            let big_enough = cfg.min_res.map(|mp| size >= mp).unwrap_or(true);

            wide_enough && tall_enough && big_enough && !is_current
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

    /// Search for wallpapers on Reddit
    pub fn search_on_reddit(config: &Configuration) -> Vec<Self> {
        let url = reddit::create_url(config);
        let mut body = String::new();
        match reqwest::get(&url) {
            Ok(r) => r,
            Err(e) => {
                error!("Could not reach reddit: {}", e);
                return Vec::new();
            }
        }.read_to_string(&mut body)
            .unwrap();

        let json = serde_json::from_str::<JsonVal>(&body[..]).unwrap();

        let mut wallpapers = json
            .get("data")
            .and_then(|data| data.get("children"))
            .map_or_else(Vec::new, |children| {
                children
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter_map(|child| child.get("data"))
                    .filter_map(|post| Wallpaper::from_json(post).ok())
                    .collect()
            });

        if config.random {
            thread_rng().shuffle(&mut wallpapers);
        }

        for wallpaper in wallpapers.iter_mut() {
            wallpaper.update_state(config.output_dir.to_owned())
                .map_err(|error| error!("Could not update state: {}", error)).ok();
        }

        wallpapers
    }

    /// Calculates the width/height ratio of this image
    pub fn ratio(&self) -> Option<f32> {
        self.dimensions
            .map(|(width, height)| width as f32 / height as f32)
    }

    pub fn megapixel(&self) -> Option<f32> {
        self.dimensions
            .map(|(width, height)| width as f32 * height as f32 / 1_000_000.0)
    }

    /// Sets this wallpaper as a background image
    pub fn set(&self) -> Result<(), String> {
        let file: Option<PathBuf> = self.file.clone();

        let file_path = file
            .ok_or_else(|| "wallpaper is not saved yet!".to_string())
            .map(|file| {
                let canonical = canonicalize(file).unwrap();
                let to_str = canonical.to_str().unwrap();
                to_str.trim_left_matches(r#"\\?\"#).to_string()
            })?;

        ::wallpaper_lib::set_from_path(&file_path)
            .map(|_| Ok(()))
            .map_err(|error| format!("could not set wallpaper {}: {}", file_path, error))?
    }

    /// Downloads this wallpaper from its [url] and computes/sets its [format] and [dimensions]
    pub fn download(&mut self) -> Result<Vec<u8>, String> {
        let mut bytes = Vec::new();
        reqwest::get(&self.url)
            .map_err(|error| format!("request failed: {}", error))?
            .read_to_end(&mut bytes)
            .map_err(|error| format!("could not read image into buffer: {}", error))?;

        self.update_with_image_data(&bytes[..])?;

        Ok(bytes)
    }

    /// Saves this wallpaper in [directory] and sets [file] to the path of the created file
    pub fn save<P: AsRef<Path>>(&mut self, dir: P, image_data: &[u8]) -> Result<(), String> {
        let dir = dir.as_ref();
        let path = self.construct_path(dir).unwrap();

        if path.is_file() {
            self.file = Some(path);
            return Ok(());
        }

        if !dir.is_dir() {
            create_dir_all(dir).map_err(|e| format!("could not create path: {}", e))?;
        }

        File::create(&path)
            .map_err(|e| format!("could not create file: {}", e))?
            .write(image_data)
            .map_err(|e| format!("could not write to file: {}", e))?;

        self.file = Some(path);

        Ok(())
    }

    /// Searches this wallpaper in [image_directory] and,
    /// if found, sets [file], [format] and [dimensions]
    fn update_state(&mut self, image_directory: String) -> Result<(), String> {
        let image_directory: &Path = image_directory.as_ref();

        let directory_content = match read_dir(image_directory) {
            Ok(content) => content,
            Err(_) => return Ok(()),
        };

        let this_filename = self.construct_filename();
        let this_filename: &str = this_filename.as_ref();

        let already_downloaded_file = directory_content
            .filter_map(|path| path.ok())
            .map(|dir_entry| dir_entry.path())
            .filter(|path| path.is_file())
            .filter(|file| {
                file.file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .starts_with(this_filename)
            })
            .last();

        match already_downloaded_file {
            Some(file) => {
                self.file = Some(file.clone());
                let mut file = File::open(file)
                    .map_err(|error| format!("Could not open file: {}", error))?;
                let mut bytes = Vec::new();
                file.read(&mut bytes)
                    .map_err(|error| format!("Could not read file: {}", error))?;
                self.update_with_image_data(&bytes[..])?;
            }
            None => (),
        };
        Ok(())
    }

    fn update_with_image_data(&mut self, data: &[u8]) -> Result<(), String> {
        let image = load_from_buf(data)
            .map_err(|error| format!("Computing dimensions failed: {}", error))?;
        let dim = image.dimensions();
        self.dimensions = Some((dim.width, dim.height));
        self.format = Some(
            match image {
                Jpeg(_) => "jpeg",
                Png(_) => "png",
                Gif(_) => "gif",
                _ => return Err("Image format not supported".to_owned()),
            }.to_owned()
        );
        Ok(())
    }

    /// The path where a wallpaper should be saved depending
    /// on its title, format and the given directory
    fn construct_path<P: AsRef<Path>>(&self, dir: P) -> Option<PathBuf> {
        let dir: &Path = dir.as_ref();
        let file_name = self.construct_filename();
        let path = dir.join(file_name);
        Some(path)
    }

    /// The name under which a wallpaper should be stored
    /// on disk depending on its title and format
    fn construct_filename(&self) -> String {
        static FORBIDDEN: [char; 9] = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];

        let new_name: String = self.title
            .trim().chars()
            .flat_map(char::to_lowercase)
            .filter(|c| !FORBIDDEN.contains(c))
            .map(|c| if c == ' ' { '_' } else { c })
            .collect();

        match &self.format {
            Some(format) => format!("{}.{}", new_name, format),
            None => new_name
        }
    }

    fn from_json(json: &JsonVal) -> Result<Self, &'static str> {
        Ok(Wallpaper {
            title: json["title"]
                .as_str()
                .ok_or("field 'title' not found")?
                .to_owned(),
            url: json["url"]
                .as_str()
                .ok_or("field 'url' not found")?
                .to_owned(),
            format: None,
            file: None,
            dimensions: None,
        })
    }
}
