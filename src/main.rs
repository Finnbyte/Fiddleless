#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
mod requests;
mod auth;
mod utils;

use auth::token::{self, read_lockfile};
use eframe::egui;
use requests::runes;
use utils::windows::{self, is_league_dir, read_cache};
use std::thread;
use std::path::{Path, PathBuf};

use crate::auth::token::{Lockfile, construct_token};
use crate::requests::state::LcuApi;
use serde::Deserialize;

const SCREEN_WIDTH: f32 = 320.0;
const SCREEN_HEIGHT: f32 = 240.0;

#[derive(Debug, Deserialize)]
pub enum Role {
    ADC, MID, TOP, JGL, SUPP, UNSURE
}

impl Role {
    fn as_str(&self) -> &'static str {
        match self {
            Role::ADC => "ADC",
            Role::MID => "MID",
            Role::TOP => "TOP",
            Role::JGL => "JGL",
            Role::SUPP => "SUPP",
            Role::UNSURE => "UNSURE"
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Champion {
    pub name: String,
    pub role: Role
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(SCREEN_WIDTH, SCREEN_HEIGHT)),
        ..Default::default()
    };

    while windows::read_cache().is_err() {
        eframe::run_native("Fiddleless Configuration", options.clone(), Box::new(|cc| Box::new(App::new(cc, AppState::CONFIGURATOR))))?
    }

    // match api.get_hovered_champion()  {
    //     Ok(Some(champion)) => println!("Hovered champ data: {}, {}", champion.name, champion.role.as_str()),
    //     Ok(None) => println!("Not in champion select!"),
    //     Err(e) => panic!("{}", e)
    // }

    eframe::run_native("Fiddleless", options, Box::new(|cc| Box::new(App::new(cc, AppState::MAIN))))?;


    Ok(())
}

#[derive(Default)]
struct Modal {
    is_open: bool,
    title: Box<str>,
    header: Box<str>,
    text: String
}

impl Modal {
    fn set_content(&mut self, new_header: &str, new_text: &str) {
        self.header = new_header.into();
        self.text = new_text.into();
    }

    fn set_visibility(&mut self, mode: bool) {
        self.is_open = mode
    }
}

enum AppState {
    CONFIGURATOR = 0,
    MAIN = 1
}

struct ConnectionRequirements {
    port: u16,
    token: String
}

#[derive(Default)]
struct App {
    api: Option<LcuApi>,
    lockfile: Option<Lockfile>,
    league_dir_path: Option<PathBuf>,
    selected_champion: Option<Champion>,
    modal: Modal
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>, state: AppState) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());

        match state {
            AppState::CONFIGURATOR => {
                Self {
                    selected_champion: None,
                    api: None,
                    lockfile: None,
                    league_dir_path: None,
                    modal: Modal { is_open: false, title: "!!!".into(), header: "".into(), text: "".into() },
                }
            }

            AppState::MAIN => {
                // Setup
                let league_dir_path = windows::read_cache().ok();
                let lockfile: Option<Lockfile> = match token::read_lockfile(league_dir_path.as_ref().unwrap().as_path()) {
                    Ok(lockfile) => Some(lockfile),
                    Err(_e) => None
                };
                let api: Option<LcuApi> = match lockfile {
                    Some(ref lockfile) => Some(LcuApi::new(&token::construct_token(&lockfile.password), lockfile.port)),
                    _ => None,
                };

                Self {
                    selected_champion: None,
                    lockfile,
                    api,
                    league_dir_path,
                    modal: Modal { is_open: false, title: "!!!".into(), header: "".into(), text: "".into() },
                }
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.modal.is_open {
            egui::Window::new(&*self.modal.title)
                .open(&mut self.modal.is_open)
                .show(ctx, |ui| {
                    ui.heading(&*self.modal.header);
                    ui.label(&self.modal.text);
                });
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.league_dir_path.is_none() {
                ui.heading("Pick your League Of Legends directory location");
                ui.label("This is needed for this software to function.");

                if ui.button("Pick folder…").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.league_dir_path = Some(path);
                    }
                }

                if let Some(picked_dir_path) = &self.league_dir_path {
                    if windows::is_league_dir(picked_dir_path.to_path_buf()) {
                        let _ = windows::create_cache(picked_dir_path);
                        _frame.close()
                    } else {
                        self.league_dir_path = None; // Not resetting this causes infinite loop of modals spawning
                        self.modal.set_content("Invalid directory given", "This directory isn't a valid LoL game folder.");
                        self.modal.set_visibility(true);
                    }
                }
            } else { // Main state
                if self.lockfile.is_none() {
                    ui.heading("League Of Legends not running");
                    ui.label("Please start League before starting Fiddleless.");
                    if ui.button("Close").clicked() {
                        _frame.close()
                    }
                    return
                } else if self.lockfile.is_some() && self.api.is_none() {
                    self.api = Some(LcuApi::new(&token::construct_token(
                        &self.lockfile.as_ref().unwrap().password), 
                        self.lockfile.as_ref().unwrap().port));
                }

                ui.heading("Welcome to Fiddleless");

                let hovered_champion_name = match self.api.as_ref().expect("API not defined").get_hovered_champion()  {
                    Ok(Some(champion)) => champion.name,
                    Ok(None) => "Not in champion select!".to_string(),
                    Err(_e) => "seksiä".into()
                };

                ui.label(hovered_champion_name);
                
                thread::sleep(std::time::Duration::from_millis(500));
            }
        });
    }
}
