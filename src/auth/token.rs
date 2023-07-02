use base64::{Engine as _, engine::general_purpose};

pub struct Lockfile {
    pub port: u16,
    pub password: String
}

pub fn read_lockfile(league_dir_path: &std::path::Path) -> Result<Lockfile, std::io::Error> {
    let contents = std::fs::read_to_string(league_dir_path.join("lockfile"))?;
    let data: Vec<&str> = contents.split(":").collect();

    let port: u16 = data[2].parse::<u16>().unwrap_or(1234);
    let password: String = String::from(data[3]);

    Ok(Lockfile { port, password })
}

pub fn construct_token(password: &String) -> String {
    let encoded_password = general_purpose::STANDARD_NO_PAD.encode(format!("riot:{}", password));
    return format!("Basic {}", encoded_password);
}
