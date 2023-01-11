use std::path::PathBuf;

use clap::{Parser, Subcommand};
use kubeman::KubeManConfig;

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
}

fn main() {
    let cli = Cli::parse();

    let kubemanconfig = match KubeManConfig::new(cli.dir, cli.kube_dir) {
        Ok(kmc) => kmc,
        Err(msg) => panic!("{}", msg),
    };

    match &cli.command {
        Some(Commands::List) => list(&kubemanconfig),
        None => (),
    }
}

fn list(kubemanconfig: &KubeManConfig) {
    let current_config = kubemanconfig.current_config();
    for kubeconfig in kubemanconfig.configs() {
        let hash = kubeconfig.hash();
        let name = match kubeconfig.name().as_ref() {
            Some(n) => String::from(n),
            None => String::from(&hash[..8]),
        };
        let mut output = format!("  {}", name);
        if let Some(cf) = current_config {
            if cf.hash() == kubeconfig.hash() {
                output = format!("* {}", name);
            }
        }
        println!("{}", output);
    }
}
