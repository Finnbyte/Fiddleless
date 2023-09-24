use reqwest::header::AUTHORIZATION;

use crate::{Champion, Role};
use tokio;
use crate::utils::champions;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct APIChampion {
    championId: u16
}

#[derive(Default)]
pub struct LcuApi {
    url: String,
    client: reqwest::blocking::Client,
    pub token: String,
    pub port: u16
}

impl LcuApi {
    pub fn new(token: &String, port: u16) -> Self {
        // Format URL of API
        let url = format!("https://127.0.0.1:{}", port.to_string());
        // Create a client
        let client_configuration: reqwest::blocking::ClientBuilder = reqwest::blocking::Client::builder().danger_accept_invalid_certs(true);
        let client: reqwest::blocking::Client = client_configuration.build().expect("Building a client failed!");
        //let client_configuration: reqwest::blocking::ClientBuilder = reqwest::Client::builder().danger_accept_invalid_certs(true);
        //let client: reqwest::blocking::Client = client_configuration.build().expect("Building a client failed!");
        // Return Self with fields
        Self {
            url,
            client,
            token: token.to_string(),
            port
        }
    }

    pub fn try_establish_connection() -> Self {
                 
    }

    pub fn get_hovered_champion(&self) -> Result<Option<Champion>, Box<dyn std::error::Error>> {
        let my_selection_url = format!("{}/lol-champ-select/v1/session/my-selection", self.url);
        // Sending a request
        let res: reqwest::blocking::Response = self.client.get(my_selection_url).header(AUTHORIZATION, &self.token).send()?;

        // Converts JSON representation to struct representation
        let ch: Option<APIChampion> = res.json().ok();
        if let Some(ch) = ch {
            let champion_name = champions::champion_id_to_name(ch.championId);
            let champ = Champion { name: champion_name.to_string(), role: Role::UNSURE };

            Ok(Some(champ))                                                                                     
        } else {
            Ok(None)
        }
    }
}


