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
use super::set_wallpaper;

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
    /// Search for up to [limit] wallpapers on reddit
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

        let mut wallpapers = json.get("data")
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
        }

        wallpapers
    }

    /// Calculates the width/height ratio of this image
    /// Returns [None] if [width] and/or [height] is [None]
    pub fn ratio(&self) -> Option<f32> {
        self.dimensions
            .map(|(width, height)| width as f32 / height as f32)
    }

    /// Sets this wallpaper as a background image
    pub fn set(&self) -> Result<(), String> {
        match self.file {
            Some(ref file) => {
                let canonical = canonicalize(file).unwrap();
                let path = canonical.to_str().unwrap();
                match set_wallpaper(path) {
                    Ok(()) => Ok(()),
                    Err(_) => Err("could not set wallpaper!".to_owned()),
                }
            }
            None => Err("wallpaper is not saved yet!".to_owned()),
        }
    }

    /// Downloads this wallpaper from its [url] and computes/sets its [format] and [dimensions]
    pub fn download(&mut self) -> Result<Vec<u8>, String> {
        let mut bytes = Vec::new();
        reqwest::get(&self.url)
            .map_err(|error| format!("request failed: {}", error))?
            .read_to_end(&mut bytes)
            .map_err(|error| format!("could not read image into buffer: {}", error))?;

        match load_from_buf(&bytes) {
            Ok(img) => {
                let dim = img.dimensions();
                self.dimensions = Some((dim.width, dim.height));
                self.format = Some(
                    match img {
                        Jpeg(_) => "jpeg",
                        Png(_) => "png",
                        Gif(_) => "gif",
                        _ => return Err("image format not supported".to_owned()),
                    }.to_owned(),
                )
            }
            Err(e) => return Err(format!("computing dimensions failed: {}", e)),
        };

        Ok(bytes)
    }

    /// Saves this wallpaper in [directory] and sets [file] to the path of the created file
    pub fn save<P: AsRef<Path>>(&mut self, directory: P, image_data: &[u8]) -> Result<(), String> {
        let folder = directory.as_ref();
        info!("Saving to {:?}", folder);
        let path = self.construct_path(folder).unwrap();

        if path.is_file() {
            self.file = Some(path);
            return Ok(());
        }

        if !folder.is_dir() {
            create_dir_all(folder).map_err(|e| format!("could not create path: {}", e))?;
        }

        File::create(&path)
            .map_err(|e| format!("could not create file: {}", e))?
            .write(image_data)
            .map_err(|e| format!("could not write to file: {}", e))?;

        self.file = Some(path);

        Ok(())
    }

    /// Searches this wallpaper in [image_directory] and, if found, sets [file] and [format]
    fn update_state(&mut self, image_directory: String) {
        if self.file.is_some() && self.format.is_some() {
            return;
        }

        let image_directory: &Path = image_directory.as_ref();

        let directory_content = match read_dir(image_directory) {
            Ok(content) => content,
            Err(_) => return,
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
                let extension = file.extension().unwrap().to_str().unwrap().to_string();
                self.file = Some(file);
                self.format = Some(extension);
            }
            None => (),
        };
    }

    fn construct_filename(&self) -> String {
        static FORBIDDEN: [char; 9] = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];

        let format = &self.format.clone().unwrap_or(String::new());

        let mut new_name: String = self
            .title
            .trim()
            .chars()
            .flat_map(|c| c.to_lowercase())
            .filter(|c| !FORBIDDEN.contains(c))
            .map(|c| if c == ' ' { '_' } else { c })
            .collect();

        new_name.push('.');
        new_name.push_str(format);
        new_name
    }

    fn construct_path<P: AsRef<Path>>(&self, folder: P) -> Option<PathBuf> {
        let file_name = self.construct_filename();
        let folder: &Path = folder.as_ref();
        Some(folder.join(file_name))
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
