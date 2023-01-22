use std::fs;
use std::io;
use std::io::Read;
use std::path::PathBuf;
use std::result;
use std::str;

use clap::{Parser, Subcommand};
use kubenv::KubEnv;

type Result<T = ()> = result::Result<T, String>;

#[derive(Parser)]
#[command(name = "KubEnv")]
#[command(version = "0.1.0")]
#[command(about = "CLI application for managing kubernetes environments")]
#[command(author = "Oleg Yurchik <oleg@yurchik.space>")]
struct Cli {
    #[arg(short, long)]
    dir: Option<PathBuf>,
    #[arg(short, long)]
    kube_dir: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    List,
    Apply {
        name: String,
    },
    Add {
        #[arg(short, long)]
        name: Option<String>,
        #[arg(short, long)]
        file: Option<PathBuf>,
    },
    Remove {
        name: String,
    },
    Export {
        name: String,
        #[arg(short, long)]
        file: Option<PathBuf>,
    },
}

fn print_error(message: String) {
    eprintln!("[ERROR] {}", message);
}

fn main() {
    let cli = Cli::parse();

    let mut kubenv = match KubEnv::new(cli.dir, cli.kube_dir) {
        Ok(ke) => ke,
        Err(msg) => {
            print_error(msg);
            return;
        }
    };
    if let Err(msg) = kubenv.sync() {
        print_error(msg);
        return;
    }

    let result = match &cli.command {
        Commands::List => list(&kubenv),
        Commands::Apply { name } => apply(&kubenv, &name),
        Commands::Add { name, file } => add(&kubenv, &name, &file),
        Commands::Remove { name } => remove(&kubenv, &name),
        Commands::Export { name, file } => export(&kubenv, &name, &file),
    };
    if let Err(msg) = result {
        print_error(msg);
    }
}

fn list(kubenv: &KubEnv) -> Result {
    let current_config = kubenv.current_config();
    for kubeconfig in kubenv.configs() {
        let name = kubeconfig.name();
        let mut output = format!("  {}", name);
        if let Some(cf) = current_config {
            if cf.hash() == kubeconfig.hash() {
                output = format!("* {}", name);
            }
        }
        println!("{}", output);
    }

    return Ok(());
}

fn apply(kubenv: &KubEnv, name: &str) -> Result {
    kubenv.apply(name)?;
    println!("Apply config '{}' succesfully", name);

    return Ok(());
}

fn remove(kubenv: &KubEnv, name: &str) -> Result {
    kubenv.remove(&name)?;
    println!("Remove config '{}' successfully", name);

    return Ok(());
}

fn add(kubenv: &KubEnv, name: &Option<String>, path: &Option<PathBuf>) -> Result {
    let content: Vec<u8> = match path {
        Some(path) => match fs::read(&path) {
            Ok(c) => c,
            Err(msg) => {
                match path.to_str() {
                    Some(p) => return Err(format!("Cannot read file '{}': {}", p, msg)),
                    None => return Err(format!("Cannot read file: {}", msg)),
                };
            }
        },
        None => {
            let mut content: Vec<u8> = Vec::new();
            if let Err(msg) = io::stdin().read_to_end(&mut content) {
                return Err(format!("Cannot read content from stdin: {}", msg));
            };
            content
        }
    };
    kubenv.import(&content, name.clone())?;
    match name {
        Some(n) => println!("Import config '{}' successfully", n),
        None => println!("Import config succesfully"),
    }

    return Ok(());
}

fn export(kubenv: &KubEnv, name: &str, path: &Option<PathBuf>) -> Result {
    let content = kubenv.export(&name)?;
    let content = match str::from_utf8(&content) {
        Ok(c) => c,
        Err(msg) => {
            return Err(format!(
                "Cannot convert '{}' config to utf-8: {}",
                name, msg
            ))
        }
    };

    match path {
        Some(path) => {
            if let Err(msg) = fs::write(path, content) {
                match path.to_str() {
                    Some(path) => return Err(format!("Cannot write file '{}': {}", path, msg)),
                    None => return Err(format!("Cannot write file: {}", msg)),
                }
            }
            println!("Config '{}' exported successfully", name);
        }
        None => println!("{}", content),
    }

    return Ok(());
}
