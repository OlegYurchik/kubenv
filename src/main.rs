use std::fs;
use std::io;
use std::io::Read;
use std::path::PathBuf;
use std::str;

use clap::{Parser, Subcommand};
use kubeman::{KubeMan, Result};

#[derive(Parser)]
#[command(name = "KubeMan")]
#[command(author = "Oleg Yurchik <oleg@yurchik.space>")]
struct Cli {
    #[arg(short, long)]
    dir: Option<PathBuf>,
    #[arg(short, long)]
    kube_dir: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
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

    let mut kubeman = match KubeMan::new(cli.dir, cli.kube_dir) {
        Ok(km) => km,
        Err(msg) => {
            print_error(msg);
            return;
        }
    };
    if let Err(msg) = kubeman.sync() {
        print_error(msg);
        return;
    }

    let result = match &cli.command {
        Some(Commands::List) => list(&kubeman),
        Some(Commands::Apply { name }) => apply(&kubeman, &name),
        Some(Commands::Add { name, file }) => add(&kubeman, &name, &file),
        Some(Commands::Remove { name }) => remove(&kubeman, &name),
        Some(Commands::Export { name, file }) => export(&kubeman, &name, &file),
        None => Ok(()),
    };
    if let Err(msg) = result {
        print_error(msg);
    }
}

fn list(kubeman: &KubeMan) -> Result {
    let current_config = kubeman.current_config();
    for kubeconfig in kubeman.configs() {
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

fn apply(kubeman: &KubeMan, name: &str) -> Result {
    kubeman.apply(name)?;
    println!("Apply config '{}' succesfully", name);

    return Ok(());
}

fn remove(kubeman: &KubeMan, name: &str) -> Result {
    kubeman.remove(&name)?;
    println!("Remove config '{}' successfully", name);

    return Ok(());
}

fn add(kubeman: &KubeMan, name: &Option<String>, path: &Option<PathBuf>) -> Result {
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
    kubeman.import(&content, name.clone())?;
    match name {
        Some(n) => println!("Import config '{}' successfully", n),
        None => println!("Import config succesfully"),
    }

    return Ok(());
}

fn export(kubeman: &KubeMan, name: &str, path: &Option<PathBuf>) -> Result {
    let content = kubeman.export(&name)?;
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
