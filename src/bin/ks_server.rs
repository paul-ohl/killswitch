use std::{collections::HashMap, io};

use rouille::{input::json_input, router, try_or_400, Response};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct JsonData {
    name: String,
}

#[derive(Deserialize, Debug)]
struct ConfigFile {
    server: ServerConfig,
    projects: HashMap<String, bool>,
}

#[derive(Deserialize, Debug)]
struct ServerConfig {
    host: String,
    port: u16,
}

const CONFIG_FILE_PATH: &str = "./config.yml";

#[derive(Clone, Debug, thiserror::Error, Deserialize)]
enum Error {
    #[error("Error reading config file: {0}")]
    ConfigFileReadError(String),
    #[error("Error parsing config file: {0}")]
    ConfigFileParseError(String),
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::ConfigFileReadError(error.to_string())
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(error: serde_yaml::Error) -> Self {
        Error::ConfigFileParseError(error.to_string())
    }
}

fn main() {
    let address = match get_config() {
        Ok(config) => format!("{}:{}", config.server.host, config.server.port),
        Err(e) => {
            eprintln!("Error reading config file: {}", e);
            return;
        }
    };

    println!("Server started at http://{}", address);
    rouille::start_server(address, move |request| {
        rouille::log(request, io::stdout(), || {
            router!(request,
                (POST) (/) => {
                    let data: JsonData = try_or_400!(json_input(request));

                    if let Ok(config) = get_config() {
                            check_project_presence(&data.name, config.projects)
                    } else {
                        Response::empty_204().with_status_code(500).with_no_cache()
                    }
                },
                _ => rouille::Response::empty_404()
            )
        })
    });
}

fn get_config() -> Result<ConfigFile, Error> {
    let config_file = std::fs::read_to_string(CONFIG_FILE_PATH)?;
    Ok(serde_yaml::from_str(&config_file)?)
}

fn check_project_presence(project_name: &str, projects: HashMap<String, bool>) -> Response {
    if let Some(is_project_valid) = projects.get(project_name) {
        if *is_project_valid {
            Response::empty_204().with_no_cache()
        } else {
            Response::empty_400().with_status_code(402).with_no_cache()
        }
    } else {
        Response::empty_404().with_no_cache()
    }
}
