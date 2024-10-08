use std::collections::HashMap;
use std::env;
use std::fs::create_dir_all;
use std::fs::File;
use std::io;
use std::io::ErrorKind::Other;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use serde_json::Map;

use crate::tournament_info::{Athlete, Belt, Club, Sender, Tournament, WeightCategory};

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


fn string_to_iso_8859_1_bytes(s: &str) -> Vec<u8> {
    s.chars().map(|c| { c as u8 }).collect()
}

#[allow(clippy::cast_possible_truncation)]
pub fn read_club(path: impl AsRef<Path>) -> io::Result<Club> {
    let mut file = File::options().read(true).open(path)?;
    let mut s = String::new();
    file.read_to_string(&mut s)?;
    let serde_value: serde_json::Value = serde_json::from_str(&s)?;
    let value = serde_value.as_object().ok_or(io::Error::new(Other, "did not find a map"))?;
    let club_name = String::from(value.get("club").ok_or(io::Error::new(Other, "club-name not provided"))?
        .as_str().ok_or(io::Error::new(Other, "club-name not a string"))?);
    let sender_given_name = String::from(value.get("given").ok_or(io::Error::new(Other, "sender's given name not provided"))?
        .as_str().ok_or(io::Error::new(Other, "given name not a string"))?);
    let sender_sur_name = String::from(value.get("sur").ok_or(io::Error::new(Other, "sender's surname not provided"))?
        .as_str().ok_or(io::Error::new(Other, "surname not a string"))?);
    let address = String::from(value.get("address").ok_or(io::Error::new(Other, "address not provided"))?
        .as_str().ok_or(io::Error::new(Other, "address not a string"))?);
    let postal_code = value.get("postal-code").ok_or(io::Error::new(Other, "postal code not provided"))?.as_u64()
        .ok_or(io::Error::new(Other, "postal code not a number"))? as u16;
    let town = String::from(value.get("town").ok_or(io::Error::new(Other, "town not provided"))?
        .as_str().ok_or(io::Error::new(Other, "town not a string"))?);
    let private = String::from(value.get("private").ok_or(io::Error::new(Other, "private phone number not provided"))?
        .as_str().ok_or(io::Error::new(Other, "private phone number not a string"))?);
    let public = String::from(value.get("public").ok_or(io::Error::new(Other, "public phone number not provided"))?
        .as_str().ok_or(io::Error::new(Other, "public phone number not a string"))?);
    let fax = String::from(value.get("fax").ok_or(io::Error::new(Other, "fax number not provided"))?
        .as_str().ok_or(io::Error::new(Other, "fax number not a string"))?);
    let mobile = String::from(value.get("mobile").ok_or(io::Error::new(Other, "mobile number not provided"))?
        .as_str().ok_or(io::Error::new(Other, "mobile number not a string"))?);
    let mail = String::from(value.get("mail").ok_or(io::Error::new(Other, "e-mail address not provided"))?
        .as_str().ok_or(io::Error::new(Other, "e-mail address not a string"))?);
    let club_number = value.get("club-number").ok_or(io::Error::new(Other, "club-number not provided"))?.as_u64()
        .ok_or(io::Error::new(Other, "club-number not a number"))?;
    let county = String::from(value.get("county").ok_or(io::Error::new(Other, "county not provided"))?
        .as_str().ok_or(io::Error::new(Other, "county not a string"))?);
    let region = String::from(value.get("region").ok_or(io::Error::new(Other, "region not provided"))?
        .as_str().ok_or(io::Error::new(Other, "region not a string"))?);
    let state = String::from(value.get("state").ok_or(io::Error::new(Other, "state not provided"))?
        .as_str().ok_or(io::Error::new(Other, "state not a string"))?);
    let group = String::from(value.get("group").ok_or(io::Error::new(Other, "group not provided"))?
        .as_str().ok_or(io::Error::new(Other, "group not a string"))?);
    let nation = String::from(value.get("nation").ok_or(io::Error::new(Other, "nation not provided"))?
        .as_str().ok_or(io::Error::new(Other, "nation not a string"))?);
    let sender = Sender::new(club_name.clone(), sender_given_name, sender_sur_name, address, postal_code, town, private, public,
        fax, mobile, mail);
    Ok(Club::new(club_name, club_number, sender, county, region, state, group, nation))
}

#[allow(clippy::cast_possible_truncation)]
pub fn read_athletes(path: impl AsRef<Path>) -> io::Result<Vec<Athlete>> {
    let mut file = File::options().read(true).open(path)?;
    let mut s = String::new();
    file.read_to_string(&mut s)?;
    let serde_value: serde_json::Value = serde_json::from_str(&s)?;
    let value = serde_value.as_array().ok_or(io::Error::new(Other, "did not find a list"))?;
    let mut ret = Vec::with_capacity(value.len());
    for serde_athlete in value {
        let athlete = serde_athlete.as_object().ok_or(io::Error::new(Other, "athlete not a map"))?;
        let athlete_given_name = String::from(athlete.get("given").ok_or(io::Error::new(Other, "athlete's given name not provided"))?
            .as_str().ok_or(io::Error::new(Other, "athlete's given name not a string"))?);
        let athlete_sur_name =  String::from(athlete.get("sur").ok_or(io::Error::new(Other, "athlete's surname not provided"))?
            .as_str().ok_or(io::Error::new(Other, "athlete's surname not a string"))?);
        let belt_str = athlete.get("belt").ok_or(io::Error::new(Other, "athlete's belt not provided"))?
            .as_str().ok_or(io::Error::new(Other, "athlete's belt not a string"))?;
        let belt = Belt::from_str(belt_str).ok_or(io::Error::new(Other, "belt format not understood"))?;
        let year = athlete.get("year").ok_or(io::Error::new(Other, "athlete's birth year not provided"))?.as_u64()
            .ok_or(io::Error::new(Other, "athlete's birth year not an integer"))? as u16;
        ret.push(Athlete::new(athlete_given_name, athlete_sur_name, year, belt, WeightCategory::default()));
    }
    Ok(ret)
}

fn write_tournament(path: impl AsRef<Path>, tournament: &Tournament) -> io::Result<()> {
    let mut file = File::options().write(true).create(true).truncate(true).open(path)?;
    file.write_all(&string_to_iso_8859_1_bytes(&tournament.render()))?;
    Ok(())
}

pub fn write_athletes(path: impl AsRef<Path>, athletes: &[Athlete]) -> io::Result<()> {
    let mut values: Vec<serde_json::Value> = Vec::with_capacity(athletes.len());
    for athlete in athletes {
        values.push(athlete.serialise());
    }

    let mut file = File::options().write(true).create(true).truncate(true).open(path)?;
    file.write_all(serde_json::to_string(&values)?.as_bytes())?;
    Ok(())
}

pub fn write_club(path: impl AsRef<Path>, club: &Club) -> io::Result<()> {
    let mut map = serde_json::Map::new();
    map.insert(String::from("club"), club.get_name().into());
    map.insert(String::from("given"), club.get_sender().get_given_name().into());
    map.insert(String::from("sur"), club.get_sender().get_sur_name().into());
    map.insert(String::from("address"), club.get_sender().get_address().into());
    map.insert(String::from("postal-code"), club.get_sender().get_postal_code().into());
    map.insert(String::from("town"), club.get_sender().get_town().into());
    map.insert(String::from("private"), club.get_sender().get_private_phone().into());
    map.insert(String::from("public"), club.get_sender().get_public_phone().into());
    map.insert(String::from("fax"), club.get_sender().get_fax().into());
    map.insert(String::from("mobile"), club.get_sender().get_mobile().into());
    map.insert(String::from("mail"), club.get_sender().get_mail().into());
    map.insert(String::from("club-number"), club.get_number().into());
    map.insert(String::from("county"), club.get_county().into());
    map.insert(String::from("region"), club.get_region().into());
    map.insert(String::from("state"), club.get_state().into());
    map.insert(String::from("group"), club.get_group().into());
    map.insert(String::from("nation"), club.get_nation().into());

    let mut file = File::options().write(true).create(true).truncate(true).open(path)?;

    file.write_all(serde_json::to_string(&map)?.as_bytes())?;
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
