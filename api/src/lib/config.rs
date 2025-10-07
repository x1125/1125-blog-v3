use std::env;
use std::path::{Path, PathBuf};

pub static CONTENT_NAMES: &'static [&str] = &["posts", "tags"];

#[derive(Debug, Clone)]
pub struct ConfigError {
    pub message: String,
}

#[derive(Clone)]
pub struct Config {
    pub working_path: PathBuf,
    // FIXME: leak is required to satisfy 'static lifetime of State
    pub token: &'static str,
}

impl Config {
    pub fn new() -> Result<Self, ConfigError> {
        let config = Config {
            working_path: path_from_env("WORKING_PATH")?,
            token: "",
        };
        Ok(config)
    }

    pub fn get_input_path(&self) -> PathBuf {
        self.working_path.join(Path::new("posts"))
    }

    pub fn get_output_path(&self) -> PathBuf {
        self.working_path.join(Path::new("p"))
    }
}

pub trait ConfigType {
    fn get_token(&self) -> String;
}

impl ConfigType for Config {
    fn get_token(&self) -> String {
        return self.token.to_owned();
    }
}

fn get_required_env(name: &str) -> Result<String, ConfigError> {
    match env::var(name) {
        Ok(env_val) => Ok(env_val),
        Err(_) => Err(ConfigError { message: format!("{} environment variable is missing", name).to_string() })
    }
}

fn path_from_env(name: &str) -> Result<PathBuf, ConfigError> {
    let relative_path = get_required_env(name)?;
    let expanded_path = shellexpand::full(&relative_path).unwrap().into_owned();
    Ok(Path::new(expanded_path.as_str()).to_owned())
}