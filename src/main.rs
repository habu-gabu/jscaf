use std::collections::HashMap;
use std::fs;
use std::io::Error;
#[derive(Debug)]
enum ProjectParseError {
    FileNotFound,
    MissingKey(String),
    IoError(Error),
}

impl From<Error> for ProjectParseError {
    fn from(err: Error) -> Self {
        ProjectParseError::IoError(err)
    }
}
use self::ProjectParseError::{FileNotFound, IoError, MissingKey};
fn main() {
    match parse_project_info() {
        Ok(info) => {
            println!("Group: {}", info["group"]);
            println!("Artifact: {}", info["artifact"]);
        }
        Err(e) => match e {
            FileNotFound => println!("proj.toml not found"),
            MissingKey(key) => println!("Missing key: {}", key),
            IoError(err) => println!("IO error: {}", err),
        },
    }
}
fn parse_project_info() -> Result<HashMap<String, String>, ProjectParseError> {
    let content = fs::read_to_string("proj.toml").map_err(|_| ProjectParseError::FileNotFound)?;

    let mut map = HashMap::new();

    for line in content.lines() {
        let line = line.trim();

        if line.starts_with("group") {
            let value = line
                .replace("group", "")
                .replace("=", "")
                .replace('"', "")
                .trim()
                .to_string();
            if value.is_empty() {
                return Err(ProjectParseError::MissingKey("group".to_string()));
            }
            map.insert("group".to_string(), value);
        }

        if line.starts_with("artifact") {
            let value = line
                .replace("artifact", "")
                .replace("=", "")
                .replace('"', "")
                .trim()
                .to_string();
            if value.is_empty() {
                return Err(ProjectParseError::MissingKey("artifact".to_string()));
            }
            map.insert("artifact".to_string(), value);
        }
    }

    if !map.contains_key("group") {
        return Err(ProjectParseError::MissingKey("group".to_string()));
    }
    if !map.contains_key("artifact") {
        return Err(ProjectParseError::MissingKey("artifact".to_string()));
    }

    Ok(map)
}
