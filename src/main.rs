use std::collections::HashMap;
use std::fs;
use std::io::Error;

#[derive(Debug)]
enum ProjectParseError {
    FileNotFound,
    MissingKey(String),
    IoError(Error),
    EmptyNamespace,
}

impl From<Error> for ProjectParseError {
    fn from(err: Error) -> Self {
        ProjectParseError::IoError(err)
    }
}

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum FileType {
    Class,
    Interface,
    Enum,
    Record,
    Checked,
    Unchecked,
}

#[derive(Parser, Debug)]
#[command(name = "jscaf")]
#[command(about = "Simple Java scaffolding CLI")]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Create a new Java file
    New {
        /// File type (interface, class, enum, record)
        #[arg(short = 't', long = "type", value_enum)]
        filetype: FileType,

        /// Fully qualified class name (e.g., service.UserService)
        #[arg(short = 'n', long = "name")]
        namespace: String,
    },
}

#[derive(Debug)]
struct JavaFileInfo {
    dir: Vec<String>,
    filename_with_ext: String,
    filetype: FileType,
}

impl JavaFileInfo {
    fn new(namespace: &str, filetype: FileType) -> Result<Self, ProjectParseError> {
        let info = parse_project_info()?;

        let group_str = info.get("group").unwrap();
        let artifact = info.get("artifact").unwrap();

        let mut dir: Vec<String> = group_str.split('.').map(|s| s.to_string()).collect();

        dir.push(artifact.to_string());

        let mut packages: Vec<String> = namespace.split('.').map(|s| s.to_string()).collect();

        let filename = packages.pop().ok_or(ProjectParseError::EmptyNamespace)?;
        dir.extend(packages);

        let filename_with_ext = format!("{}.java", filename);

        Ok(Self {
            dir,
            filename_with_ext,
            filetype,
        })
    }
}

use self::ProjectParseError::{EmptyNamespace, FileNotFound, IoError, MissingKey};
fn main() {
    let args = Args::parse();

    match &args.command {
        Commands::New {
            filetype,
            namespace,
        } => {
            let java_file = JavaFileInfo::new(namespace, *filetype);

            if let Err(err) = java_file {
                match err {
                    FileNotFound => panic!("proj.toml not found: {:?}", err),
                    MissingKey(key) => panic!("missing key: {:?}", key),
                    IoError(err) => panic!("io error: {:?}", err),
                    EmptyNamespace => panic!("io error: {:?}", err),
                }
            }

            println!("{:#?}", java_file.unwrap());
        }
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
