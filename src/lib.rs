use std::cmp::{Ordering, PartialOrd};
use std::fs;
use std::path::PathBuf;

use home;
use sha256;

fn get_default_kube_dir() -> Result<PathBuf, String> {
    let home_dir = match home::home_dir() {
        Some(path) => path.to_path_buf(),
        None => return Err(String::from("Cannot get home directory")),
    };

    let kube_dir = home_dir.join(".kube");
    return Ok(kube_dir);
}

fn get_default_kubeman_dir() -> Result<PathBuf, String> {
    let kube_dir = get_default_kube_dir()?;
    let kubeman_dir = kube_dir.join("kubeman");
    return Ok(kubeman_dir);
}

#[derive(PartialEq)]
pub struct KubeConfig {
    name: Option<String>,
    path: PathBuf,
    hash: String,
}

impl KubeConfig {
    pub fn new(path: PathBuf, name: Option<String>) -> Result<Self, String> {
        let hash = match sha256::try_digest(path.as_path()) {
            Ok(bytes) => bytes,
            Err(_) => match path.to_str() {
                Some(ps) => {
                    return Err(format!(
                        "Cannot read and/or getting hash from file '{}'",
                        ps,
                    ));
                }
                None => return Err(String::from("Cannot get PathBuf str")),
            },
        };

        return Ok(Self { name, path, hash });
    }

    pub fn name(&self) -> &Option<String> {
        return &self.name;
    }

    pub fn hash(&self) -> &str {
        return &self.hash;
    }
}

impl PartialOrd for KubeConfig {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if let Some(name_1) = self.name() {
            if let Some(name_2) = other.name() {
                return Some(name_1.cmp(name_2));
            }
        }

        return Some(self.hash().cmp(other.hash()));
    }
}

pub struct KubeManConfig {
    kube_dir: PathBuf,
    kubeman_dir: PathBuf,
    current_config: Option<KubeConfig>,
    configs: Vec<KubeConfig>,
}

impl KubeManConfig {
    pub fn new(kubeman_dir: Option<PathBuf>, kube_dir: Option<PathBuf>) -> Result<Self, String> {
        let kube_dir = match kube_dir {
            Some(path_buf) => path_buf,
            None => get_default_kube_dir()?,
        };
        let kubeman_dir = match kubeman_dir {
            Some(path_buf) => path_buf,
            None => get_default_kubeman_dir()?,
        };
        let mut kubemanconfig = Self {
            kube_dir,
            kubeman_dir,
            current_config: None,
            configs: vec![],
        };
        kubemanconfig.sync()?;
        return Ok(kubemanconfig);
    }

    pub fn configs(&self) -> Vec<&KubeConfig> {
        return self.configs.iter().collect();
    }

    pub fn current_config(&self) -> &Option<KubeConfig> {
        return &self.current_config;
    }

    fn sync(&mut self) -> Result<(), String> {
        let kubeman_dir = self.kube_dir.join("kubeman");
        if !kubeman_dir.is_dir() {
            if let Err(_) = fs::create_dir_all(kubeman_dir.as_path()) {
                match kubeman_dir.to_str() {
                    Some(path) => return Err(format!("Cannot create directory '{}'", path)),
                    None => return Err(String::from("Cannot create directory")),
                };
            };
        };

        self.update_configs(&kubeman_dir)?;
        self.update_current_config()?;

        return Ok(());
    }

    fn update_configs(&mut self, kubeman_dir: &PathBuf) -> Result<(), String> {
        let config_files = match fs::read_dir(kubeman_dir.clone()) {
            Ok(value) => value,
            Err(_) => match kubeman_dir.to_str() {
                Some(path) => return Err(format!("Cannot read files from directory '{}'", path)),
                None => return Err(String::from("Cannot read files from kubeman directory")),
            },
        };

        self.configs.clear();
        for config_file in config_files {
            let config_file = match config_file {
                Ok(cf) => cf,
                Err(_) => continue,
            };
            let path = config_file.path();
            let file_name = match config_file.file_name().to_str() {
                Some(ps) => String::from(ps),
                None => continue,
            };
            if path.is_file() && file_name.ends_with(".kubeconfig") {
                let name = match file_name.strip_suffix(".kubeconfig") {
                    Some(s) => String::from(s),
                    None => file_name.clone(),
                };
                match KubeConfig::new(path, Some(name)) {
                    Ok(kubeconfig) => {
                        let len = self.configs.len();
                        let mut index = 0;
                        if len > 0 {
                            index = len - 1;
                            while index > 0 && kubeconfig < self.configs[index - 1] {
                                index -= 1;
                            }
                        }
                        self.configs.insert(index, kubeconfig);
                    }
                    Err(_) => continue,
                };
            };
        }

        return Ok(());
    }

    fn update_current_config(&mut self) -> Result<(), String> {
        let current_config_file = self.kube_dir.join("config");

        if current_config_file.is_file() {
            self.current_config = Some(KubeConfig::new(current_config_file, None)?);
        }

        return Ok(());
    }
}
