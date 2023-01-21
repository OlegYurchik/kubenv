use std::fs;
use std::path::PathBuf;

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
    Remove {
        name: String,
    },
    Import {
        #[arg(short, long)]
        name: Option<String>,
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
        Some(Commands::List) => list(kubeman),
        Some(Commands::Apply { name }) => apply(kubeman, name.clone()),
        Some(Commands::Import { name, file }) => import(kubeman, name.clone(), file.clone()),
        Some(Commands::Remove { name }) => remove(kubeman, name.clone()),
        None => Ok(()),
    };
    if let Err(msg) = result {
        print_error(msg);
    }
}

fn list(kubeman: KubeMan) -> Result {
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

fn apply(kubeman: KubeMan, name: String) -> Result {
    kubeman.apply(&name)?;
    println!("Apply config '{}' succesfully", name);

    return Ok(());
}

fn import(kubeman: KubeMan, name: Option<String>, path: Option<PathBuf>) -> Result {
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
            let content: Vec<u8> = Vec::new();
            content
        }
    };
    kubeman.import(&content, name)?;
    // TODO: Need add name to println
    println!("Import config succesfully");

    return Ok(());
}

fn remove(kubeman: KubeMan, name: String) -> Result {
    kubeman.remove(name.clone())?;
    println!("Remove config '{}' successfully", name);

    return Ok(());
}
