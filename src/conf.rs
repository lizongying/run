use crate::error::Error;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::fmt;
use std::{env, fs};

#[derive(Debug, Serialize, Deserialize)]
pub struct Conf {
    pub env: String,
    pub jobs: Vec<Job>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Job {
    pub label: String,
    pub cmd: Option<String>,
    pub default_option: Option<usize>,
    pub options: Option<Vec<OptionItem>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OptionItem {
    pub label: String,
    pub cmd: String,
}

impl fmt::Display for Conf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Environment: {}\nJobs:\n", self.env)?;
        for job in &self.jobs {
            writeln!(f, "  {}", job)?;
        }
        Ok(())
    }
}

impl fmt::Display for Job {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Job Label: {}\n", self.label)?;
        if let Some(ref cmd) = self.cmd {
            write!(f, "  Command: {}\n", cmd)?;
        }
        if let Some(ref options) = self.options {
            write!(f, "  Options:\n")?;
            for option in options {
                writeln!(f, "    {}", option)?;
            }
        }
        Ok(())
    }
}

impl fmt::Display for OptionItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Option Label: {}, Command: {}", self.label, self.cmd)
    }
}

pub fn load_config() -> Result<Conf, Error> {
    let home = match env::var("HOME") {
        Ok(value) => value,
        Err(_) => {
            return Err(Error::HomeNotFound);
        }
    };

    let path = format!("{}/.run.yml", home);
    let config = match fs::read_to_string(&path) {
        Ok(value) => value,
        Err(_) => {
            if let Err(e) = fs::File::create(&path) {
                eprintln!("Error creating config file: {}", e);
                return Err(Error::IoError(e));
            }

            let config = r#"
env:
jobs:
  - label: Who am I
    cmd: who am i

  - label: Which
    default_option: 1
    options:
      - label: node
        cmd: which node

      - label: python
        cmd: which python

      - label: golang
        cmd: which golang

      - label: rust
        cmd: which rust

      - label: java
        cmd: which java

      - label: kotlin
        cmd: which kotlin
            "#
            .trim();
            if let Err(e) = fs::write(path, config) {
                eprintln!("Error writing config file: {}", e);
                return Err(Error::IoError(e));
            }

            return parse_config(config.to_string());
        }
    };
    parse_config(config)
}

fn parse_config(config: String) -> Result<Conf, Error> {
    let mut parsed_config: Conf = match serde_yaml::from_str(&config) {
        Ok(value) => value,
        Err(e) => {
            println!("Error parsing config: {}", e);
            return Err(Error::ConfigParseError);
        }
    };
    parsed_config.jobs.extend(vec![
        Job {
            label: String::from("Info"),
            cmd: None,
            default_option: None,
            options: None,
        },
        Job {
            label: String::from("Exit"),
            cmd: None,
            default_option: None,
            options: None,
        },
    ]);
    Ok(parsed_config)
}
