#![windows_subsystem = "windows"]

mod tournament_info;
mod ui;
mod utils;

use std::fs::{create_dir_all, File};
use std::io;
use std::io::Write;

use cosmic::iced::Size;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;

use utils::{crash, get_config_dir, get_config_file, get_configs, get_default_configs, get_translations, read_athletes, read_club, DEFAULT_WINDOW_SIZE};
#[cfg(not(feature="unstable"))]
use utils::{update_translations, write_language, DEFAULT_TRANSLATIONS_DE, DEFAULT_TRANSLATIONS_EN};
use ui::EMelderApp;
use crate::tournament_info::Club;

#[allow(clippy::too_many_lines)]
fn main() -> cosmic::iced::Result {
    let stdout_logger = ConsoleAppender::builder().build();
    let file_logger = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{level} from {module} on {date(%a, %Y-%m-%d at %H:%M:%S%z)}: {message}\n")))
        .build(get_config_dir().unwrap_or_else(|_err| {
            crash()
        }).join("e-melder/e-melder.log")).unwrap_or_else(
            |_err| {
                crash()
            }
        );
    let config = log4rs::Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout_logger)))
        .appender(Appender::builder().build("file", Box::new(file_logger)))
        .logger(Logger::builder()
            .appenders(["stdout", "file"])
            .build("e-melder", log::LevelFilter::Info))
        .build(Root::builder().appenders(["stdout", "file"]).build(log::LevelFilter::Info)).unwrap_or_else(|_err| {
            crash()
        });
    log4rs::init_config(config).unwrap_or_else(|_err| {
        crash()
    });
    log::info!("New run of the app");

    let config_file = match get_config_file() {
        Ok(config_file) => config_file,
        Err(err) => {
            log::error!("failed to get config-file, due to {err}");
            crash();
        }
    };

    if !config_file.exists() {
        if let Some(config_file_parent) = config_file.parent() {
            match create_dir_all(config_file_parent) {
                Ok(()) => {}
                Err(err) => {
                    log::error!("failed to create necessary directories for config-file, due to {err}");
                    crash();
                }
            }
        }

        let mut config_file = match File::options().write(true).create_new(true).open(config_file) {
            Ok(config_file) => config_file,
            Err(err) => {
                log::error!("failed to create config-file, due to {err}");
                crash();
            }
        };

        let (default_configs, tournament_basedir) = match get_default_configs() {
            Ok(default_configs) => default_configs,
            Err(err) => {
                log::error!("failed to create default-configs, due to {err}");
                crash();
            }
        };

        match config_file.write_all(default_configs.as_bytes()) {
            Ok(()) => {},
            Err(err) => {
                log::warn!("failed to write default-configs, due to {err}");
            }
        }

        #[cfg(not(feature="unstable"))]
        let lang_dir = match get_config_dir() {
            Ok(config_dir) => config_dir,
            Err(err) => {
                log::error!("failed to get config-directory, due to {err}");
                crash();
            }
        }.join("e-melder/lang");

        #[cfg(not(feature="unstable"))]
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

        match create_dir_all(tournament_basedir) {
            Ok(()) => {},
            Err(err) => {
                log::warn!("failed to create necessary directories for tournament-basedir, due to {err}");
            }
        }
    }


    #[cfg(not(feature="unstable"))]
    match update_translations() {
        Ok(()) => {}
        Err(err) => {
            log::warn!("failed to update translations, due to {err}");
        }
    };

    let mut configs = get_configs().unwrap_or_else(|err| {
        log::error!("failed to load configs, due to {err}");
        crash();
    });
    #[cfg(not(feature = "unstable"))]
    let lang_file = match get_config_dir() {
        Ok(lang_file) => lang_file,
        Err(err) => {
            log::error!("failed to get config dir, due to {err}");
            crash();
        }
    }.join("e-melder").join("lang").join(format!("{}.json", configs.lang));

    #[cfg(not(feature = "unstable"))]
    if !lang_file.exists() {
        match create_dir_all(lang_file.parent().expect("unreachable")) {
            Ok(()) => {},
            Err(err) => {
                log::error!("failed to create necessary directories for lang-file, due to {err}");
                crash();
            }   
        }

        let mut lang_file = match File::options().write(true).create_new(true).open(lang_file) {
            Ok(lang_file) => lang_file,
            Err(err) => {
                log::error!("failed to create lang-file, due to {err}");
                crash();
            }
        };

        let lang = configs.lang.as_str();
        let translations = match lang {
            "de" => DEFAULT_TRANSLATIONS_DE,
            "en" => DEFAULT_TRANSLATIONS_EN,
            // other languages, that might be supported in the future, would be listed here
            _ => "{}"
        };

        match lang_file.write_all(translations.as_bytes()) {
            Ok(()) => {},
            Err(err) => {
                log::error!("failed to write default language, due to {err}");
                crash();
            }
        }
    }

    let languages = std::fs::read_dir(get_config_dir().expect("already used it, this should not panic")
        .join("e-melder").join("lang")).unwrap_or_else(|err| {
        log::error!("failed to read config-directory/e-melder/lang, due to {err}");
        crash();
    }).map(|entry| {
        entry.unwrap_or_else(|err| {
            log::error!("failed to read config-directory/e-melder/lang, due to {err}");
            crash();
        }).path().file_stem().expect("unreachable").to_str().expect("unreachable").to_owned()
    }).collect();

    configs.langs = languages;
    let translations = get_translations(&configs.lang).unwrap_or_default();
    let club = read_club(&configs.club_file).unwrap_or_else(|err| {
        if err.kind() == io::ErrorKind::NotFound {
            Club::default()
        }
        else {
            log::warn!("failed to read club, due to {err}");
            Club::default()
        }
    });
    let athletes = read_athletes(&configs.athletes_file).unwrap_or_else(|err| {
        if err.kind() == io::ErrorKind::NotFound {
            Vec::new()
        }
        else {
            log::warn!("failed to read athletes, due to {err}");
            Vec::new()
        }
    });

    let settings = cosmic::app::settings::Settings::default()
        .theme(
            configs.theme.into()
        )
        .size(Size::from(DEFAULT_WINDOW_SIZE));

    cosmic::app::run::<EMelderApp>(settings, (configs, translations, club, athletes))
}
