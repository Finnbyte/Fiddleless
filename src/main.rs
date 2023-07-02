#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
mod requests;
mod auth;
mod utils;

use iced::futures;
use iced::widget::{self, column, container, image, row, text};
use iced::{
    Alignment, Application, Color, Command, Element, Length, Settings, Theme,
};

use auth::token::{self, read_lockfile};
use utils::windows::{self, read_cache};
use std::thread;
use std::path::{Path, PathBuf};

use crate::auth::token::{Lockfile, construct_token};
use crate::requests::state::LcuApi;
use serde::Deserialize;

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

impl Champion {
    fn view(&self) -> Element<Message> {
        column![text(format!("{}", self.name)).size(40),]
            .width(Length::Shrink)

    }
}

const SCREEN_WIDTH: f32 = 320.0;
const SCREEN_HEIGHT: f32 = 240.0;

fn main() -> iced::Result {
    while windows::read_cache().is_err() {
        eframe::run_native("Fiddleless Configuration", options.clone(), Box::new(|cc| Box::new(App::new(cc, AppState::CONFIGURATOR))))?
    }


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

// State
enum Fiddleless {
    Loading,
    Loaded { api: LcuApi },
    LeagueNotRunningError,
    ApiConnectionFailed,
    CurrentlyHovering { champion: Champion },
}

// Events
enum Message {
    ApiConnection(Result<LcuApi, Box<dyn std::error::Error>>),
    PickedChampion(Result<Champion, Box<dyn std::error::Error>>),
}

//#[derive(Default)]
//struct App {
//    api: Option<LcuApi>,
//    lockfile: Option<Lockfile>,
//    league_dir_path: Option<PathBuf>,
//    selected_champion: Option<Champion>,
//    modal: Modal
//}

enum FiddlelessError {
    LeagueNotRunningError,
    ApiConnectionFailed,
}

impl Application for Fiddleless {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Fiddleless, Command<Message>) {
        (
            Fiddleless::Loading,
            Command::perform(LcuApi::try_establish_connection(), Message::Loaded),
        )
    }

    fn title(&self) -> String {
        let subtitle = match self {
            Fiddleless::Loading => "Loading",
            Fiddleless::Loaded { api, .. } => &api.token,
            Fiddleless::LeagueNotRunningErrorL { .. } => "Hey!",
            Fiddleless::ApiConnectionFailed { .. } => "Whoops! An error occured while trying to communicate with LCU!",
        };

        format!("{subtitle} - Fiddleless")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::PickedChampion(Ok(champion)) => {
                *self = Fiddleless::Champion { champion };

                Command::none()
            }
            Message::ApiConnection(Ok(api)) => {
                *self = Fiddleless::Loaded { api }
            }
            Message::ApiConnection(Err(e)) => {
                *self = match e {
                    FiddlelessError::LeagueNotRunningError => Self::LeagueNotRunningError,
                    FiddlelessError::ApiConnectionFailed => Self::ApiConnectionFailed,
                };

                Command::none()
            }
            Message::Search => match self {
                Fiddleless::Loading => Command::none(),
                _ => {
                    *self = Fiddleless::Loading;

                    Command::perform(LcuApi::try_establish_connection(), )
                }
            },
        }
    }

    fn view(&self) -> Element<Message> {
        let content = match self {
            Fiddleless::Loading => {
                column![text("Attempting connection with League Client...").size(40),]
                    .width(Length::Shrink)
            }
            Fiddleless::Loaded { api } => column![
                column![text("Connection with LCU is established. Token: ", api.token)]
                button("Keep searching!").on_press(Message::Search)
            ]
            .max_width(500)
            .spacing(20)
            .align_items(Alignment::End),
            Fiddleless::CurrentlyHovering { champion } => column![
                champion.view()
            ]
            .max_width(500)
            .spacing(20)
            .align_items(Alignment::End),
            Fiddleless::LeagueNotRunningError => column![
                text("League of Legends is not running! Start League and press the bottom below").size(40),
                button("Restart this app")
            ]
            .spacing(20)
            .align_items(Alignment::End),
            Fiddleless::ApiConnectionFailed => column![
                text("An error occured while trying to communicate with LCU!").size(40),
            ]
        };

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
