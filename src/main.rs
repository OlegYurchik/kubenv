use std::fs;
use std::io::{stdin, stdout, BufReader, Read, Write};
use std::path::PathBuf;
use std::result;
use std::str;

use clap::{Parser, Subcommand};
use kubenv::KubEnv;

type Result<T = ()> = result::Result<T, String>;
const BUF_SIZE: usize = 1024;

#[derive(Parser)]
#[command(name = "KubEnv")]
#[command(version = "0.3.2")]
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
    Show {
        name: String,
    },
    Export {
        name: String,
        #[arg(short, long)]
        file: PathBuf,
    },
}

fn print_error(message: String) {
    eprintln!("[ERROR] {}", message);
}

fn reader_to_writer(reader: &mut dyn Read, writer: &mut dyn Write) -> Result {
    let mut buffer = vec![0; BUF_SIZE];
    let mut read_result = reader.read(&mut buffer);
    while let Ok(count) = read_result {
        if count == 0 {
            break;
        }
        if let Err(msg) = writer.write(&buffer[..count]) {
            return Err(format!("Cannot write: {}", msg));
        };
        read_result = reader.read(&mut buffer);
    }
    if let Err(msg) = read_result {
        return Err(format!("Cannot read: {}", msg));
    }

    return Ok(());
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
        Commands::Show { name } => show(&kubenv, &name),
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
    let mut reader: BufReader<Box<dyn Read>> = match path {
        Some(path) => match fs::File::open(path) {
            Ok(f) => BufReader::with_capacity(BUF_SIZE, Box::new(f)),
            Err(msg) => {
                match path.to_str() {
                    Some(p) => return Err(format!("Cannot open file '{}': {}", p, msg)),
                    None => return Err(format!("Cannot open file: {}", msg)),
                };
            }
        },
        None => BufReader::with_capacity(BUF_SIZE, Box::new(stdin())),
    };
    kubenv.set_content(name.clone(), &mut reader)?;
    match name {
        Some(n) => println!("Import config '{}' successfully", n),
        None => println!("Import config succesfully"),
    }

    return Ok(());
}

fn show(kubenv: &KubEnv, name: &str) -> Result {
    let mut reader = kubenv.get_content(name)?;
    let mut writer = stdout().lock();

    reader_to_writer(&mut reader, &mut writer)?;

    return Ok(());
}

fn export(kubenv: &KubEnv, name: &str, path: &PathBuf) -> Result {
    let mut reader = kubenv.get_content(name)?;
    let mut writer = match fs::File::create(path) {
        Ok(f) => f,
        Err(msg) => match path.to_str() {
            Some(path) => return Err(format!("Cannot open file '{}': {}", path, msg)),
            None => return Err(format!("Cannot open file: {}", msg)),
        },
    };

    reader_to_writer(&mut reader, &mut writer)?;

    println!("Config '{}' exported successfully", name);
    return Ok(());
}
