use std::collections::HashMap;
use std::env;
#[cfg(not(feature="unstable"))]
use std::fs::create_dir_all;
use std::fs::File;
use std::io;
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};

use notify_rust::Timeout;
use serde::Deserialize;
use serde_json::Map;

use crate::tournament_info::{Athlete, Belt, Club, GenderCategory, Tournament, WeightCategory};
use crate::ui::app::{Configs, OldConfigs, Theme};

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
#[cfg(target_os="windows")]
static ILLEGAL_CHARS: &str = "<>:\"/\\|?*\0";
#[cfg(not(target_os="windows"))]
static ILLEGAL_CHARS: &str = "/\0";
pub const DEFAULT_BIRTH_YEAR: u16 = 2010;
pub const DEFAULT_WINDOW_SIZE: [f32; 2] = [1450.0, 700.0];
pub const GENDERS: [GenderCategory; 3] = [GenderCategory::Mixed, GenderCategory::Male, GenderCategory::Female];
pub const BELTS: [Belt; 19] = [Belt::Kyu9, Belt::Kyu8, Belt::Kyu7, Belt::Kyu6, Belt::Kyu5,
Belt::Kyu4, Belt::Kyu3, Belt::Kyu2, Belt::Kyu1, Belt::Dan1, Belt::Dan2, Belt::Dan3, Belt::Dan4,
Belt::Dan5, Belt::Dan6, Belt::Dan7, Belt::Dan8, Belt::Dan9, Belt::Dan10];
pub const THEMES: [Theme; 3] = [Theme::System, Theme::Light, Theme::Dark];
lazy_static::lazy_static! {
    pub static ref LEGAL_GENDER_CATEGORIES: enum_map::EnumMap<GenderCategory, &'static [GenderCategory]> = enum_map::enum_map! {
        GenderCategory::Female => &[GenderCategory::Female, GenderCategory::Mixed],
        GenderCategory::Male => &[GenderCategory::Male, GenderCategory::Mixed],
        GenderCategory::Mixed => &[GenderCategory::Female, GenderCategory::Male, GenderCategory::Mixed] as &[_]
    };
}
lazy_static::lazy_static! {
    pub static ref LANG_NAMES: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("de", "Deutsch");
        m.insert("en", "English");
        m
    };
}

#[macro_export]
macro_rules! translate {
    ($translation_key:expr,$translations:expr) => {
        {
            match $crate::utils::translate_fn($translation_key,$translations) {
                Some(translation) => translation.to_owned(),
                None => {
                    log::warn!("failed to get translation");
                    $translation_key.to_owned()
                }
            }
        }
    };
}

#[macro_export]
macro_rules! translate_raw {
    ($translation_key:expr) => {
        {
            match $crate::utils::get_configs() {
                Ok(configs) => {
                    match $crate::utils::get_translations(&configs.lang) {
                        Ok(translations) => $crate::utils::translate!($translation_key, &translations),
                        Err(err) => {
                            log::warn!("failed to get translation, due to {err}");
                            $translation_key.to_owned()
                        }
                    }
                },
                Err(err) => {
                    log::warn!("failed to get translation, due to {err}");
                    $translation_key.to_owned()
                }
            }
        }
    };
}

pub use translate;

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

fn replace_illegal_chars(s: &str) -> String {
    s.replace(|c| ILLEGAL_CHARS.contains(c), "_")
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
        if path.is_empty() {
            home::home_dir().ok_or(io::Error::new(io::ErrorKind::NotFound, "could not locate config directory"))
        }
        else {
            Ok(PathBuf::from(path))
        }
    }
    else {
        home::home_dir().ok_or(io::Error::new(io::ErrorKind::NotFound, "could not locate config directory"))
    }
}

pub fn get_config_file() -> io::Result<PathBuf> {
    let base_dir = get_config_dir()?;
    Ok(base_dir.join("e-melder/config.json"))
}

pub fn translate_fn<'a>(translation_key: &str, translations: &'a HashMap<String, String>) -> Option<&'a str> {
    translations.get(translation_key).map(String::as_str)
}

pub fn write_tournaments(tournaments: &[Tournament], configs: &Configs) -> io::Result<()> {
    if tournaments.is_empty() {
        return Ok(());
    }
    let tournament_base_value = &configs.tournament_basedir;
    let tournament_base = PathBuf::from(tournament_base_value);
    
    for tournament in tournaments {
        let path = tournament_base.join(format!("{}{} ({}).dm4", replace_illegal_chars(tournament.get_name()),
            replace_illegal_chars(tournament.get_age_category()), tournament.get_gender_category().render()));
        write_tournament(path, tournament)?;
    }

    Ok(())
}

pub fn write_configs(configs: &Configs) -> io::Result<()> {
    let config_file = get_config_file()?;
    let file = File::options().write(true).truncate(true).open(&config_file)?;
    serde_json::to_writer(file, configs).map_err(Into::into)
}

pub fn get_configs() -> io::Result<Configs> {
    let latest_version_path = match get_config_dir() {
        Ok(config_dir) => config_dir,
        Err(err) => {
            log::error!("failed to get config-directory, due to {err}");
            crash();
        }
    }.join("e-melder/latest");
    let config_file = get_config_file()?;
    let file = File::options().read(true).open(config_file)?;
    #[allow(clippy::if_not_else)]
    if !latest_version_path.exists() {
        let old_configs: OldConfigs = serde_json::from_reader(file)?;
        let configs = old_configs.into();
        write_configs(&configs)?;
        Ok(configs)
    }
    else {
        let mut latest_version_file = File::options().read(true).open(&latest_version_path)?;
        let mut latest_version = String::with_capacity(6);
        latest_version_file.read_to_string(&mut latest_version)?;
        if latest_version.starts_with("1.") || latest_version.starts_with("2.") || latest_version.starts_with("3.") {
            let old_configs: OldConfigs = serde_json::from_reader(file)?;
            let mut latest_version_file = File::options().write(true).truncate(true).open(&latest_version_path)?;
            latest_version_file.write_all(VERSION.as_bytes())?;
            let configs = old_configs.into();
            write_configs(&configs)?;
            return Ok(configs);
        }
        else if latest_version != VERSION {
            let mut latest_version_file = File::options().write(true).truncate(true).open(&latest_version_path)?;
            latest_version_file.write_all(VERSION.as_bytes())?;
        }
        serde_json::from_reader(file).map_err(Into::into)
    }
}

pub fn get_default_configs() -> io::Result<(String, PathBuf)> {
    let athletes_file = get_config_dir()?.join("e-melder").join("athletes.json");
    let club_file = get_config_dir()?.join("e-melder").join("club.json");
    let tournament_basedir = home::home_dir().ok_or(io::Error::other("users does not have a home-directory"))?.join("e-melder");
    let mut default_config = Map::new();
    default_config.insert(String::from("lang"), "de".into());
    default_config.insert(String::from("theme"), "System".into());
    default_config.insert(String::from("club-file"), club_file.to_str().expect("unreachable").into());
    default_config.insert(String::from("athletes-file"), athletes_file.to_str().expect("unreachable").into());
    default_config.insert(String::from("tournament-basedir"), tournament_basedir.to_str().expect("unreachable").into());
    default_config.insert(String::from("default-gender"), "g".into());
    Ok((serde_json::to_string(&default_config).expect("unreachable"), tournament_basedir))
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
    let _ = std::thread::spawn(|| {
        #[cfg(all(target_family="unix", not(target_os="macos")))]
        let _ = notify_rust::Notification::new()
        .summary("E-Melder")
        .body(&format!("An unrecoverable error occurred, please look into the logs to see what happened.\n{}{}",
        "If you think this is a bug, please file a bug report at ", CODE_LINK))
        .sound_name("dialog-error")
        .timeout(Timeout::Never)
        .show().map(|handle| handle.wait_for_action(|_| {}));
        #[cfg(not(all(target_family="unix", not(target_os="macos"))))]
        let _ = notify_rust::Notification::new()
        .summary("E-Melder")
        .body(&format!("An unrecoverable error occurred, please look into the logs to see what happened.\n{}{}",
        "If you think this is a bug, please file a bug report at ", CODE_LINK))
        .timeout(Timeout::Never)
        .show();
    }).join();
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
                log::warn!("failed to create necessary directories for lang-files, due to {err}");
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
        latest_version_file.read_to_string(&mut latest_version)?;
        if latest_version != VERSION {
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
                    log::warn!("failed to create necessary directories for lang-files, due to {err}");
                }
            }

            drop(latest_version_file);
        }
    }

    Ok(())
}

pub fn get_translations(lang: &str) -> io::Result<HashMap<String, String>> {
    let lang_file_name = get_config_dir()?.join("e-melder").join("lang").join(format!("{lang}.json"));
    let lang_file = File::options().read(true).open(lang_file_name)?;
    serde_json::from_reader(lang_file).map_err(Into::into)
}

#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn serialize_gender<S>(gender_category: &GenderCategory, serializer: S) -> Result<S::Ok, S::Error>
where S: serde::Serializer {
    serializer.serialize_str(gender_category.render())
}

pub fn deserialize_gender<'de, D>(deserializer: D) -> Result<GenderCategory, D::Error>
where D: serde::Deserializer<'de> {
    GenderCategory::from_str(&String::deserialize(deserializer)?).ok_or(serde::de::Error::custom("Invalid Gender category"))
}

#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn serialize_weight_category<S>(weight_category: &WeightCategory, serializer: S) -> Result<S::Ok, S::Error>
where S: serde::Serializer {
    serializer.serialize_str(&weight_category.to_string())
}

pub fn deserialize_weight_category<'de, D>(deserializer: D) -> Result<WeightCategory, D::Error>
where D: serde::Deserializer<'de> {
    WeightCategory::from_str(&String::deserialize(deserializer)?).ok_or(serde::de::Error::custom("Invalid Weight category"))
}
