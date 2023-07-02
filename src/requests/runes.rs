use reqwest;

use crate::Champion;

pub fn get_recommended_runes(champ: &Champion) -> Result<(), reqwest::Error> {
    let res = reqwest::blocking::get("https://www.op.gg/champions/kaisa/adc/runes?region=global&tier=platinum_plus")?;
    Ok(())
}
