use crate::error::Error;
use std::env;
use std::process::Command;

pub struct Shell {
    shell: String,
}

impl Shell {
    pub fn new() -> Result<Self, Error> {
        let shell = match env::var("SHELL") {
            Ok(value) => value,
            Err(_) => {
                return Err(Error::ShellNotFound);
            }
        };

        Ok(Shell { shell })
    }

    pub fn get_shell(&self) -> String {
        self.shell.clone()
    }

    pub fn execute(&mut self, cmd: String) -> Result<String, Error> {
        let output = match Command::new(self.shell.as_str())
            .arg("-c")
            .arg(cmd)
            .output()
        {
            Ok(value) => value,
            Err(_) => {
                return Err(Error::ExecutionError);
            }
        };

        let stdout_output = if !output.stdout.is_empty() {
            String::from_utf8_lossy(&output.stdout).to_string()
        } else {
            String::new()
        };

        let stderr_output = if !output.stderr.is_empty() {
            String::from_utf8_lossy(&output.stderr).to_string()
        } else {
            String::new()
        };

        Ok(format!("{}{}", stdout_output, stderr_output))
    }
}
