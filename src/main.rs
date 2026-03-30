use clap::{Parser, Subcommand, ValueEnum};
use std::collections::HashMap;
use std::fs;
use std::io::{Error, Write};
use std::path::Path;

const JAVA_SRC_DIR: &str = "src/main/java";

enum ErrorType {
    FileNotFound,
    MissingKey(String),
    IoError,
    EmptyNamespace,
    Conflict(String),
}

impl From<Error> for ErrorType {
    fn from(_: Error) -> Self {
        ErrorType::IoError
    }
}

use self::ErrorType::*;

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
        /// File-Type
        #[arg(short = 't', long = "type", value_enum)]
        filetype: FileType,

        /// Fully qualified class name (e.g., service.UserService)
        #[arg(short = 'n', long = "name")]
        namespace: String,
    },
}

fn parse_project_info() -> Result<HashMap<String, String>, ErrorType> {
    let content = fs::read_to_string("proj.toml").map_err(|_| ErrorType::FileNotFound)?;
    let mut map = HashMap::new();

    for line in content.lines() {
        let line = line.trim();

        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim().trim_matches('"');

            if value.is_empty() {
                return Err(ErrorType::MissingKey(key.to_string()));
            }

            match key {
                "group" | "artifact" => {
                    map.insert(key.to_string(), value.to_string());
                }
                _ => {}
            }
        }
    }

    if !map.contains_key("group") {
        return Err(ErrorType::MissingKey("group".to_string()));
    }
    if !map.contains_key("artifact") {
        return Err(ErrorType::MissingKey("artifact".to_string()));
    }

    Ok(map)
}

#[derive(Debug)]
struct JavaFileInfo {
    dir: Vec<String>,
    filename_with_ext: String,
    filetype: FileType,
}

impl JavaFileInfo {
    fn new(namespace: &str, filetype: FileType) -> Result<Self, ErrorType> {
        let info = parse_project_info()?;

        let group_str = info.get("group").unwrap();
        let artifact = info.get("artifact").unwrap();

        let mut dir: Vec<String> = group_str.split('.').map(|s| s.to_string()).collect();

        dir.push(artifact.to_string());

        let mut packages: Vec<String> = namespace.split('.').map(|s| s.to_string()).collect();

        let filename = packages.pop().ok_or(ErrorType::EmptyNamespace)?;
        dir.extend(packages);

        let filename_with_ext = format!("{}.java", filename);

        Ok(Self {
            dir,
            filename_with_ext,
            filetype,
        })
    }

    fn create_file(&self) -> Result<(), ErrorType> {
        let dir_path = Path::new(JAVA_SRC_DIR);
        let full_path = self
            .dir
            .iter()
            .fold(dir_path.to_path_buf(), |acc, d| acc.join(d));

        fs::create_dir_all(&full_path)?;

        let file_path = full_path.join(&self.filename_with_ext);

        if file_path.exists() {
            return Err(ErrorType::Conflict(file_path.to_string_lossy().to_string()));
        }

        let mut file = fs::File::create(&file_path)?;

        file.write_all(self.create_boilerplate().as_bytes())?;

        Ok(())
    }

    fn create_boilerplate(&self) -> String {
        let package_name = self.dir.join(".");
        let class_name = self.filename_with_ext.trim_end_matches(".java");

        let boilerplate = match self.filetype {
            FileType::Class => format!(
                "package {};\n\npublic class {} {{\n\n}}",
                package_name, class_name
            ),
            FileType::Interface => format!(
                "package {};\n\npublic interface {} {{\n\n}}",
                package_name, class_name
            ),
            FileType::Enum => format!(
                "package {};\n\npublic enum {} {{\n\n}}",
                package_name, class_name
            ),
            FileType::Record => format!(
                "package {};\n\npublic record {}() {{\n\n}}",
                package_name, class_name
            ),
            FileType::Checked => format!(
                "package {};\n\npublic class {} extends Exception {{\n\
     \n    public {}() {{ super(); }}\n\
     \n    public {}(String message) {{ super(message); }}\n\
     \n    public {}(String message, Throwable cause) {{ super(message, cause); }}\n\
     \n    public {}(Throwable cause) {{ super(cause); }}\n\
     \n}}",
                package_name, class_name, class_name, class_name, class_name, class_name
            ),

            FileType::Unchecked => format!(
                "package {};\n\npublic class {} extends RuntimeException {{\n\
     \n    public {}() {{ super(); }}\n\
     \n    public {}(String message) {{ super(message); }}\n\
     \n    public {}(String message, Throwable cause) {{ super(message, cause); }}\n\
     \n    public {}(Throwable cause) {{ super(cause); }}\n\
     \n}}",
                package_name, class_name, class_name, class_name, class_name, class_name
            ),
        };

        boilerplate
    }
}

fn main() {
    let args = Args::parse();

    match &args.command {
        Commands::New {
            filetype,
            namespace,
        } => {
            let java_file = JavaFileInfo::new(namespace, *filetype).unwrap_or_else(|err| {
                match err {
                    FileNotFound => eprintln!("proj.toml file khai ta banako!"),
                    MissingKey(key) => eprintln!("{} rakhna xuttais hau!", key),
                    EmptyNamespace => eprintln!("file ko naam ta sahi de na ho!"),
                    _ => eprintln!("k padkyo lau feri!"),
                }
                std::process::exit(1);
            });

            if let Err(ErrorType::Conflict(filename)) = java_file.create_file() {
                eprintln!("duita file eutai naam ko rakhna bhayena hau: {}", filename);
                std::process::exit(1);
            }

            println!("La Dami Dami!")
        }
    }
}
