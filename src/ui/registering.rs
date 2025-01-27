use egui::{TextWrapMode, Ui};
use egui_extras::{Column, TableBuilder};

use crate::tournament_info::{registering_athletes_to_tournaments, GenderCategory, RegisteringAthlete};
use crate::utils::{write_tournaments, translate};
use super::EMelderApp;

enum Written {
    Successful,
    Error,
    InvalidWeightCategory
}

#[allow(clippy::too_many_lines, clippy::module_name_repetitions)]
pub fn show_registering(app: &mut EMelderApp, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.label(translate!("register.name", &app.translations));
        ui.text_edit_singleline(&mut app.registering.name);
    });

    ui.horizontal(|ui| {
        ui.label(translate!("register.place", &app.translations));
        ui.text_edit_singleline(&mut app.registering.place);
    });

    ui.horizontal(|ui| {
        ui.label(translate!("register.date", &app.translations));
        ui.add(egui_extras::DatePickerButton::new(&mut app.registering.date).format("%d.%m.%Y"));
    });

    if ui.button(translate!("register.register", &app.translations)).clicked() {
        let tournaments = registering_athletes_to_tournaments(
            &app.registering.athletes, &app.registering.name, app.registering.date,
            &app.registering.place, &app.club);
        
        let written = if let Some(tournaments) = tournaments {
            match write_tournaments(&tournaments, &app.config) {
                Ok(()) => {
                    Written::Successful
                }
                Err(err) => {
                    log::warn!("failed to write tournaments, due to {err}");
                    Written::Error
                }
            }
        } else { Written::InvalidWeightCategory };

        match written {
            Written::Successful => {
                let tournament_basedir = app.config.tournament_basedir.clone();
                #[cfg(all(target_family="unix", not(target_os="macos")))]
                let translations = app.translations.clone();
                #[cfg(all(target_family="unix", not(target_os="macos")))]
                std::thread::spawn(move || {
                    let _ = notify_rust::Notification::new()
                    .summary(&translate!("application.title", &translations))
                    .body(&translate!("register.notification.ask", &translations))
                    .sound_name("dialog-question")
                    .action("yes", &translate!("register.notification.yes", &translations))
                    .action("no", &translate!("register.notification.no", &translations))
                    .show().map(|handle| {
                        handle.wait_for_action(|action| {
                            if action == "yes" {
                                let _ = open::that_detached(tournament_basedir);
                            }
                        });
                    });
                });

                #[cfg(any(not(target_family="unix"), target_os="macos"))]
                let _ = open::that_detached(tournament_basedir);
            }
            Written::Error => {
                let translations = app.translations.clone();
                std::thread::spawn(move || {
                    #[cfg(all(target_family="unix", not(target_os="macos")))]
                    let _ = notify_rust::Notification::new()
                    .summary(&translate!("application.title", &translations))
                    .body(&translate!("register.notification.io_error", &translations))
                    .sound_name("dialog-error")
                    .show().map(|handle| handle.wait_for_action(|_| {}));
                    #[cfg(not(all(target_family="unix", not(target_os="macos"))))]
                    let _ = notify_rust::Notification::new()
                    .summary(&translate!("application.title", &translations))
                    .body(&translate!("register.notification.io_error", &translations))
                    .show();
                });
            }
            Written::InvalidWeightCategory => {
                let translations = app.translations.clone();
                std::thread::spawn(move || {
                    #[cfg(all(target_family="unix", not(target_os="macos")))]
                    let _ = notify_rust::Notification::new()
                    .summary(&translate!("application.title", &translations))
                    .body(&translate!("register.notification.invalid_weight_category", &translations))
                    .sound_name("dialog-error")
                    .show().map(|handle| handle.wait_for_action(|_| {}));
                    #[cfg(not(all(target_family="unix", not(target_os="macos"))))]
                    let _ = notify_rust::Notification::new()
                    .summary(&translate!("application.title", &translations))
                    .body(&translate!("register.notification.invalid_weight_category", &translations))
                    .show();
                });
            }
        }
    }

    ui.separator();

    show_table_registering_adding(app, ui);

    ui.separator();

    if app.registering.athletes.is_empty() {
        ui.label(translate!("register.table.empty", &app.translations));
    }
    else {
        show_table_registering(app, ui);
    }
}

#[allow(clippy::too_many_lines)]
fn show_table_registering(app: &mut EMelderApp, ui: &mut Ui) {
    let mut to_delete = None;
    ui.push_id("register.table.register", |ui| {
        let table = TableBuilder::new(ui)
            .columns(Column::auto().at_least(100.0), 7)
            .column(Column::auto().at_least(50.0));

        table.header(20.0, |mut header| {
            header.col(|ui| {
                ui.strong(translate!("register.table.given_name", &app.translations));
            });
            header.col(|ui| {
                ui.strong(translate!("register.table.sur_name", &app.translations));
            });
            header.col(|ui| {
                ui.strong(translate!("register.table.belt", &app.translations));
            });
            header.col(|ui| {
                ui.strong(translate!("register.table.year", &app.translations));
            });
            header.col(|ui| {
                ui.strong(translate!("register.table.gender_category", &app.translations));
            });
            header.col(|ui| {
                ui.strong(translate!("register.table.age_category", &app.translations));
            });
            header.col(|ui| {
                ui.strong(translate!("register.table.weight_category", &app.translations));
            });
            header.col(|_ui| {});
        }).body(|mut body| {
            for (index, athlete) in app.registering.athletes.iter_mut().enumerate() {
                body.row(18.0, |mut row| {
                    row.col(|ui| {
                        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                        ui.label(athlete.get_given_name());
                    });
                    row.col(|ui| {
                        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                        ui.label(athlete.get_sur_name());
                    });
                    row.col(|ui| {
                        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                        ui.label(translate!(&format!("add.belt.{}", athlete.get_belt().serialise()), &app.translations));
                    });
                    row.col(|ui| {
                        ui.label(athlete.get_birth_year().to_string());
                    });
                    row.col(|ui| {
                        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                        egui::ComboBox::from_id_salt(index)
                        .selected_text(translate!(&format!("register.table.gender_category.{}", athlete.get_gender_category().render()), &app.translations))
                        .show_ui(ui, |ui| {
                            for gender_category in [GenderCategory::Mixed, GenderCategory::Female, GenderCategory::Male] {
                                ui.selectable_value(athlete.get_gender_category_mut(), gender_category,
                                    translate!(&format!("register.table.gender_category.{}", gender_category.render()), &app.translations));
                            }
                        });
                    });
                    row.col(|ui| {
                        ui.text_edit_singleline(athlete.get_age_category_mut());
                    });
                    row.col(|ui| {
                        ui.text_edit_singleline(athlete.get_weight_category_mut());
                    });
                    row.col(|ui| {
                        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                        if ui.button(translate!("register.table.delete", &app.translations)).clicked() {
                            to_delete = Some(index);
                        }
                    });
                });
            }
        });
    });

    if let Some(index) = to_delete {
        app.registering.athletes.remove(index);
    }
}

#[allow(clippy::too_many_lines)]
fn show_table_registering_adding(app: &mut EMelderApp, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.label(translate!("register.search", &app.translations));
        ui.text_edit_singleline(&mut app.registering.search);
    });

    let mut athletes_shown = false;
    ui.push_id("register.table.add", |ui| {
        let table = TableBuilder::new(ui).columns(Column::auto().at_least(100.0), 4)
            .column(Column::auto().at_least(50.0)).max_scroll_height(100.0);

        table.header(20.0, |mut header| {
            header.col(|ui| {
                ui.strong(translate!("register.table.given_name", &app.translations));
            });
            header.col(|ui| {
                ui.strong(translate!("register.table.sur_name", &app.translations));
            });
            header.col(|ui| {
                ui.strong(translate!("register.table.belt", &app.translations));
            });
            header.col(|ui| {
                ui.strong(translate!("register.table.year", &app.translations));
            });
        }).body(|mut body| {
            for athlete in &app.athletes {
                if !matches_query(&format!("{} {}", athlete.get_given_name(), athlete.get_sur_name()), &app.registering.search) {
                    continue;
                }
                athletes_shown = true;

                body.row(18.0, |mut row| {
                    row.col(|ui| {
                        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                        ui.label(athlete.get_given_name());
                    });
                    row.col(|ui| {
                        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                        ui.label(athlete.get_sur_name());
                    });
                    row.col(|ui| {
                        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                        ui.label(translate!(&format!("add.belt.{}", athlete.get_belt().serialise()), &app.translations));
                    });
                    row.col(|ui| {
                        ui.label(athlete.get_birth_year().to_string());
                    });
                    row.col(|ui| {
                        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                        if ui.button(translate!("register.table.add", &app.translations)).clicked() {
                            app.registering.athletes.push(RegisteringAthlete::from_athlete(athlete,
                                app.config.default_gender_category));
                        }
                    });
                });
            }
        });
    });

    if !athletes_shown {
        ui.label(translate!("register.search.empty", &app.translations));
    }
}

fn matches_query(base: &str, query: &str) -> bool {
    // value for comparison was obtained by testing various values and choosing
    // the values with the results that felt best
    base.contains(query) || textdistance::nstr::jaro(base, query) >= 0.65
}
