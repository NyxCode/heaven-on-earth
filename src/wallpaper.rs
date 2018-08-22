use super::serde_json::Value as JsonVal;
use std::path::{Path, PathBuf};
use std::fs::{File, create_dir_all, canonicalize};
use super::reqwest;
use std::error::Error;
use std::io::{Read, Write};
use std::fs::read_dir;
use super::serde_json;
use super::immeta::{GenericMetadata::*, load_from_buf};
use super::set_wallpaper;
use super::reddit;

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
    pub fn search_on_reddit(mode: &reddit::Mode, limit: u8) -> Vec<Self> {
        // assemble url
        let mut url = mode.to_url();
        if url.contains("?") { url.push('&') } else { url.push('?') }
        url.push_str("limit=");
        url.push_str(&limit.to_string());

        let mut body = String::new();
        match reqwest::get(&url) {
            Ok(r) => r,
            Err(e) => {
                error!("Could not reach reddit: {}", e.description());
                return Vec::new();
            }
        }.read_to_string(&mut body).unwrap();

        let json = serde_json::from_str::<JsonVal>(&body[..]).unwrap();

        let posts = match json.get("data") {
            None => Vec::new(),
            Some(data) => match data.get("children") {
                None => Vec::new(),
                Some(children) => children
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter_map(|child| child.get("data"))
                    .collect()
            }
        };

        posts.iter()
            .map(|post| Wallpaper::from_json(post))
            .filter_map(|res| res.ok())
            .collect()
    }

    /// Calculates the width/height ratio of this image
    /// Returns [None] if [width] and/or [height] is [None]
    pub fn ratio(&self) -> Option<f32> {
        self.dimensions.map(|(width, height)| width as f32 / height as f32)
    }

    /// Sets this wallpaper as a background image
    pub fn set(&self) -> Result<(), String> {
        match self.file {
            Some(ref file) => {
                let canonical = canonicalize(file).unwrap();
                let path = canonical.to_str().unwrap();
                match set_wallpaper(path) {
                    Ok(()) => Ok(()),
                    Err(_) => Err("could not set wallpaper!".to_owned())
                }
            }
            None => Err("wallpaper is not saved yet!".to_owned())
        }
    }

    /// Downloads this wallpaper from its [url] and computes/sets its [format] and [dimensions]
    pub fn download(&mut self) -> Result<Vec<u8>, String> {
        let mut bytes = Vec::<u8>::new();
        match reqwest::get(&self.url) {
            Err(e) => return Err(["request failed: ", e.description()].concat()),
            Ok(r) => r
        }.read_to_end(&mut bytes).expect("could not read image into buffer");

        match load_from_buf(&bytes) {
            Ok(img) => {
                let dim = img.dimensions();
                self.dimensions = Some((dim.width, dim.height));
                self.format = Some(match img {
                    Jpeg(_) => "jpeg",
                    Png(_) => "png",
                    Gif(_) => "gif",
                    _ => return Err("image format not supported".to_owned())
                }.to_owned())
            }
            Err(e) => return Err(["computing dimensions failed: ", e.description()].concat())
        };

        Ok(bytes)
    }

    /// Saves this wallpaper in [directory] and sets [file] to the path of the created file
    pub fn save<P: AsRef<Path>>(&mut self, directory: P, image_data: &[u8]) -> Result<(), String> {
        let folder = directory.as_ref();
        let path = self.construct_path(folder).unwrap();

        if path.is_file() {
            self.file = Some(path);
            return Ok(());
        }

        if !folder.is_dir() {
            create_dir_all(folder).unwrap()
        }

        match File::create(&path) {
            Ok(mut file) => match file.write(image_data) {
                Ok(_) => {
                    self.file = Some(path);
                    Ok(())
                }
                Err(e) => Err(["could not write to file: ", e.description()].concat())
            },
            Err(e) => Err(["could not create file: ", e.description()].concat())
        }
    }

    /// Searches this wallpaper in [image_directory] and, if found, sets [file] and [format]
    pub fn update_state<P: AsRef<Path>>(&mut self, image_directory: P) {
        if self.file.is_some() && self.format.is_some() {
            return;
        }

        let image_directory: &Path = image_directory.as_ref();

        let directory_content = match read_dir(image_directory) {
            Ok(content) => content,
            Err(_) => return
        };

        let this_filename = self.construct_filename();
        let this_filename: &str = this_filename.as_ref();

        let already_downloaded_file = directory_content
            .filter_map(|path| path.ok())
            .map(|dir_entry| dir_entry.path())
            .filter(|path| path.is_file())
            .filter(|file| {
                let name = {
                    let path = file.clone();
                    path.file_name().unwrap().to_str().unwrap().to_owned()
                };
                name.starts_with(this_filename)
            }).last();

        match already_downloaded_file {
            Some(file) => {
                let extension = file.extension().unwrap().to_str().unwrap().to_string();
                self.file = Some(file);
                self.format = Some(extension);
            }
            None => ()
        };
    }


    fn construct_filename(&self) -> String {
        static FORBIDDEN: [char; 9] = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];

        let format = &self.format.clone().unwrap_or("".to_owned());
        let name = (&self.title).trim().to_lowercase();
        let mut new_name = String::with_capacity(name.len());
        for character in name.chars() {
            if !FORBIDDEN.contains(&character) {
                if character == ' ' {
                    new_name.push('_')
                } else {
                    new_name.push(character)
                }
            }
        };

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
            title: match json["title"].as_str() {
                Some(t) => t.to_owned(),
                None => return Err("field 'title' not found")
            },
            url: match json["url"].as_str() {
                Some(t) => t.to_owned(),
                None => return Err("field 'url' not found")
            },
            format: None,
            file: None,
            dimensions: None,
        })
    }
}


