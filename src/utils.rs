use std::collections::HashMap;
use std::env;
#[cfg(not(feature="unstable"))]
use std::fs::create_dir_all;
use std::fs::File;
use std::io;
use std::io::ErrorKind::Other;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use serde_json::Map;

use crate::tournament_info::{Athlete, Club, Tournament};

#[cfg(not(feature = "unstable"))]
pub static DEFAULT_TRANSLATIONS_DE: &str = include_str!("../lang/de.json");
#[cfg(not(feature = "unstable"))]
pub static DEFAULT_TRANSLATIONS_EN: &str = include_str!("../lang/en.json");

#[cfg(not(feature = "unstable"))]
pub static VERSION: &str = env!("CARGO_PKG_VERSION");
#[cfg(feature = "unstable")]
pub static VERSION: &str = "unstable";
pub static LICENSE: &str = "GNU GPL v2";
pub static LICENSE_LINK: &str = "https://github.com/UchiWerfer/e-melder-gui/blob/master/LICENSE";
pub static CODE_LINK: &str = "https://github.com/UchiWerfer/e-melder-gui";
static API_LINK: &str = "https://api.github.com/repos/UchiWerfer/e-melder-gui/releases/latest";

pub fn read_athletes(path: impl AsRef<Path>) -> io::Result<Vec<Athlete>> {
    let athletes_file = File::options().read(true).open(path)?;
    Ok(serde_json::from_reader(athletes_file)?)
}

pub fn write_athletes(path: impl AsRef<Path>, athletes: &[Athlete]) -> io::Result<()> {
    let athletes_file = File::options().write(true).create(true).truncate(true).open(path)?;
    Ok(serde_json::to_writer(athletes_file, athletes)?)
}

pub fn read_club(path: impl AsRef<Path>) -> io::Result<Club> {
    let club_file = File::options().read(true).open(path)?;
    Ok(serde_json::from_reader(club_file)?)
}

pub fn write_club(path: impl AsRef<Path>, club: &Club) -> io::Result<()> {
    let club_file = File::options().write(true).create(true).truncate(true).open(path)?;
    Ok(serde_json::to_writer(club_file, club)?)
}

fn string_to_iso_8859_1_bytes(s: &str) -> Vec<u8> {
    s.chars().map(|c| { c as u8 }).collect()
}

fn write_tournament(path: impl AsRef<Path>, tournament: &Tournament) -> io::Result<()> {
    let mut file = File::options().write(true).create(true).truncate(true).open(path)?;
    file.write_all(&string_to_iso_8859_1_bytes(&tournament.render()))?;
    Ok(())
}

#[cfg(target_os="linux")]
pub fn get_config_dir() -> io::Result<PathBuf> {
    // try using $XDG_CONFIG_HOME, otherwise use ~/.config
    let xdg_config = env::var("XDG_CONFIG_HOME");
    if let Ok(path) = xdg_config {
        if path.is_empty() {
            Ok(home::home_dir().ok_or(io::Error::new(io::ErrorKind::NotFound, "could not locate config directory"))?
            .join(".config"))
        }
        else {
            Ok(PathBuf::from(path))
        }
    }
    else {
        Ok(home::home_dir().ok_or(io::Error::new(io::ErrorKind::NotFound, "could not locate config directory"))?
            .join(".config"))
    }
}

#[cfg(not(target_os="linux"))]
pub fn get_config_dir() -> io::Result<PathBuf> {
    // try using %APPDATA%, use %HOME% instead
    let app_data = env::var("APPDATA");
    if let Ok(path) = app_data {
        Ok(PathBuf::from(path))
    }
    else {
        home::home_dir().ok_or(io::Error::new(io::ErrorKind::NotFound, "could not locate config directory"))
    }
}

pub fn get_config_file() -> io::Result<PathBuf> {
    let base_dir = get_config_dir()?;
    Ok(base_dir.join("e-melder/config.json"))
}

pub fn get_config(config: &str) -> io::Result<serde_json::Value> {
    let config_file = get_config_file()?;
    let mut file = File::options().read(true).open(config_file)?;
    let mut s = String::new();
    file.read_to_string(&mut s)?;
    let parsed = serde_json::from_str::<serde_json::Value>(&s)?;
    let configs = parsed.as_object().ok_or(
        io::Error::new(Other, "could not read configs"))?;
    let config_value = configs.get(config).ok_or(io::Error::new(Other, "did not find config"))?;
    Ok(config_value.to_owned())
}

pub fn translate_fn(translation_key: &str) -> io::Result<String> {
    let lang = String::from(get_config("lang")?.as_str().ok_or(
        io::Error::new(Other, "lang-config not a string")
    )?) + ".json";
    let lang_file_name = get_config_dir()?.join("e-melder").join("lang").join(lang);
    let mut lang_file = File::options().read(true).open(lang_file_name)?;
    let mut s = String::new();
    lang_file.read_to_string(&mut s)?;
    let parsed: serde_json::Value = serde_json::from_str(&s)?;
    let translations = parsed.as_object().ok_or(
        io::Error::new(Other, "could not read configs"))?;
    let translation = translations.get(translation_key);
    Ok(String::from(translation.map_or(Ok(translation_key), |translation| {
        translation.as_str().ok_or(io::Error::other("translation not a string"))
    })?))
}

pub fn write_tournaments(tournaments: &[Tournament]) -> io::Result<()> {
    if tournaments.is_empty() {
        return Ok(());
    }
    let tournament_base_value = get_config("tournament-basedir")?;
    let tournament_base = PathBuf::from(tournament_base_value.as_str().ok_or(io::Error::new(Other,
        "tournament-basedir not a string"))?);
    
    for tournament in tournaments {
        let path = tournament_base.join(format!("{}{} ({}).dm4", tournament.get_name(), tournament.get_age_category(),
            tournament.get_gender_category().render()));
        write_tournament(path, tournament)?;
    }

    Ok(())
}

pub fn write_config(config: &str, value: serde_json::Value) -> io::Result<()> {
    let config_file = get_config_file()?;
    let mut file_read = File::options().read(true).open(&config_file)?;
    let mut s = String::new();
    file_read.read_to_string(&mut s)?;
    let mut parsed: serde_json::Value = serde_json::from_str(&s)?;
    let configs = parsed.as_object_mut().ok_or(
        io::Error::new(Other, "could not read configs"))?;
    configs.insert(config.to_owned(), value);
    drop(file_read);
    let mut file_write = File::options().write(true).truncate(true).open(&config_file)?;
    file_write.write_all(serde_json::to_string(&configs)?.as_bytes())
}

#[macro_export]
macro_rules! translate {
    ($translation_key:expr) => {
        {
            match $crate::utils::translate_fn($translation_key) {
                Ok(translation) => translation,
                Err(err) => {
                    log::warn!("failed to get translation, due to {err}");
                    $translation_key.to_owned()
                }
            }
        }
    };
}

pub use translate;

lazy_static::lazy_static! {
    pub static ref LANG_NAMES: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("de", "Deutsch");
        m.insert("en", "English");
        m
    };
}

pub fn get_default_config() -> io::Result<(String, PathBuf)> {
    let athletes_file = get_config_dir()?.join("e-melder").join("athletes.json");
    let club_file = get_config_dir()?.join("e-melder").join("club.json");
    let tournament_basedir = home::home_dir().ok_or(io::Error::other("users does not have a home-directory"))?.join("e-melder");
    let mut default_config = Map::new();
    default_config.insert(String::from("lang"), "de".into());
    default_config.insert(String::from("dark-mode"), false.into());
    default_config.insert(String::from("club-file"), club_file.to_str().expect("unreachable").into());
    default_config.insert(String::from("athletes-file"), athletes_file.to_str().expect("unreachable").into());
    default_config.insert(String::from("tournament-basedir"), tournament_basedir.to_str().expect("unreachable").into());
    default_config.insert(String::from("default-gender-category"), "g".into());
    Ok((serde_json::to_string(&default_config).expect("unreachable"), tournament_basedir))
}

#[derive(Debug)]
pub enum UpdateAvailability {
    UpdateAvailable,
    NoUpdateAvailable,
    RunningUnstable
}

impl From<bool> for UpdateAvailability {
    fn from(value: bool) -> Self {
        if value { Self::UpdateAvailable }
        else { Self::NoUpdateAvailable }
    }
}

pub fn check_update_available(current_version: &str) -> io::Result<UpdateAvailability> {
    if current_version == "unstable" {
        return Ok(UpdateAvailability::RunningUnstable);
    }
    let body = reqwest::blocking::Client::builder().user_agent("").build().map_err(|err| {
        io::Error::other(err)
    })?.get(API_LINK).send().map_err(|err| {
        io::Error::other(err)
    })?.text().map_err(|err| {
        io::Error::other(err)
    })?;
    let parsed: serde_json::Value = serde_json::from_str(&body)?;
    let version_value = parsed.get("tag_name").ok_or(io::Error::other("did not get \"tag_name\" attribute in api-response"))?;
    let version = version_value.as_str().ok_or(io::Error::other("\"tag_name\" attribute is not a string"))?;
    Ok(((String::from("v") + current_version) != version).into())
}

#[cfg(not(feature="unstable"))]
pub fn write_language(language: &str, translations: &str) -> io::Result<()> {
    let lang_file_path = get_config_dir()?.join("e-melder/lang").join(format!("{language}.json"));
    let mut lang_file = File::options().read(false).write(true).truncate(true).create(true).open(lang_file_path)?;
    lang_file.write_all(translations.as_bytes())
}

pub fn crash() -> ! {
    let _ = notify_rust::Notification::new()
    .summary("E-Melder")
    .body(&format!("An unrecoverable error occurred, please look into the logs to see what happened.\n{}{}",
    "If you think this is a bug, please file a bug report at ", CODE_LINK))
    .sound_name("dialog-error")
    .show();
    panic!()
}

#[cfg(not(feature="unstable"))]
pub fn update_translations() -> io::Result<()> {
    let latest_version_path = match get_config_dir() {
        Ok(config_dir) => config_dir,
        Err(err) => {
            log::error!("failed to get config-directory, due to {err}");
            crash();
        }
    }.join("e-melder/latest");

    #[allow(clippy::if_not_else)]
    if !latest_version_path.exists() {
        let lang_dir = get_config_dir()?.join("e-melder/lang");

        match create_dir_all(lang_dir) {
            Ok(()) => {
                match write_language("en", DEFAULT_TRANSLATIONS_EN) {
                    Ok(()) => {}
                    Err(err) => {
                        log::warn!("failed to write english-translations, due to {err}");
                    }
                }
                match write_language("de", DEFAULT_TRANSLATIONS_DE) {
                    Ok(()) => {}
                    Err(err) => {
                        log::warn!("failed to write german-translation, due to {err}");
                    }

                }
            }
            Err(err) => {
                log::warn!("failed to create neccessary directories for lang-files, due to {err}");
                return Ok(());
            }
        }

        let mut latest_version_file =  File::options().create(true).write(true).truncate(true)
            .open(&latest_version_path)?;
        latest_version_file.write_all(VERSION.as_bytes())?;
    }
    else {
        let mut latest_version_file = File::options().read(true).open(&latest_version_path)?;
        // x.y.z usually requires 5 bytes, one per '.' and one each for x, y and z.
        // 1 extra bytes in case of unexpectedly long versions
        let mut latest_version = String::with_capacity(6);
        dbg!(latest_version_file.read_to_string(&mut latest_version)?);
        if latest_version != VERSION {
            dbg!(latest_version);
            let lang_dir = get_config_dir()?.join("e-melder/lang");
                    
            match create_dir_all(lang_dir) {
                Ok(()) => {
                    match write_language("en", DEFAULT_TRANSLATIONS_EN) {
                        Ok(()) => {}
                        Err(err) => {
                            log::warn!("failed to write english-translations, due to {err}");
                        }
                    }
                    match write_language("de", DEFAULT_TRANSLATIONS_DE) {
                        Ok(()) => {}
                        Err(err) => {
                            log::warn!("failed to write german-translation, due to {err}");
                        }
    
                    }
                }
                Err(err) => {
                    log::warn!("failed to create neccessary directories for lang-files, due to {err}");
                }
            }

            drop(latest_version_file);
            let mut latest_version_file = File::options().write(true).truncate(true).open(&latest_version_path)?;
            latest_version_file.write_all(VERSION.as_bytes())?;
        }
    }

    Ok(())
}
