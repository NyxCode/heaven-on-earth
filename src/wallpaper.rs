use super::serde_json::Value as JsonVal;
use std::path::{Path, PathBuf};
use std::fs::{File, create_dir_all, canonicalize};
use super::reqwest;
use std::error::Error;
use std::io::{Read, Write};
use super::serde_json;
use super::immeta::{GenericMetadata::*, load_from_buf};
use super::set_wallpaper;
use super::reddit;

#[derive(Debug, Clone)]
pub struct Wallpaper {
    pub title: String,
    pub url: String,
    pub format: Option<String>,
    pub file: Option<PathBuf>,
    pub dimensions: Option<(u32, u32)>,
}

fn get_string(obj: &JsonVal, key: &str) -> Option<String> {
    obj.get(key).and_then(|x| x.as_str()).map(|x| x.to_string())
}

impl Wallpaper {
    pub fn from_json(json: &JsonVal) -> Result<Self, &'static str> {
        let url = match get_string(&json, "url") {
            Some(t) => t,
            None => return Err("field 'url' not found")
        };

        Ok(Wallpaper {
            title: match get_string(&json, "title") {
                Some(t) => t,
                None => return Err("field 'title' not found")
            },
            url,
            format: None,
            file: None,
            dimensions: None,
        })
    }

    /// Searches for up to [limit] wallpapers on reddit
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

    pub fn is_saved<P: AsRef<Path>>(&mut self, folder: P) -> bool {
        let folder: &Path = folder.as_ref();
        if !folder.is_dir() {
            return false
        }
        for path in super::std::fs::read_dir(folder).unwrap() {
            let path = path.unwrap().path();
            if path.is_file() {
                let expected_name = self.construct_filename();
                let expected_name: &str = expected_name.as_ref();
                let name = {
                    let path = path.clone();
                    path.file_name().unwrap().to_str().unwrap().to_owned()
                };
                if name.starts_with(expected_name) {
                    let extension = path.extension().unwrap().to_str().unwrap().to_string();
                    self.file = Some(path);
                    self.format = Some(extension);
                    return true
                }
            }
        }
        false
    }

    pub fn save<P: AsRef<Path>>(&mut self, folder: P, data: &[u8]) -> Result<(), String> {
        let folder = folder.as_ref();
        let path = self.construct_path(folder).unwrap();

        if path.is_file() {
            self.file = Some(path);
            return Ok(());
        }

        if !folder.is_dir() {
            create_dir_all(folder).unwrap()
        }

        match File::create(&path) {
            Ok(mut file) => match file.write(data) {
                Ok(_) => {
                    self.file = Some(path);
                    Ok(())
                }
                Err(e) => Err(["could not write to file: ", e.description()].concat())
            },
            Err(e) => Err(["could not create file: ", e.description()].concat())
        }
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

    pub fn ratio(&self) -> Option<f32> {
        self.dimensions.map(|(width, height)| width as f32 / height as f32)
    }

    pub fn set(&self) -> Result<(), String> {
        match self.file {
            Some(ref file) => {
                let canonical = canonicalize(file).unwrap();
                let path = canonical.to_str().unwrap();
                match set_wallpaper(path) {
                    Ok(()) => Ok(()),
                    Err(e) => Err(["could not set wallpaper: ", e.description()].concat())
                }
            }
            None => Err("wallpaper is not saved yet!".to_owned())
        }
    }
}


