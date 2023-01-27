use std::cmp::{Ordering, PartialOrd};
use std::fs;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;
use std::result;

use hashbrown::HashMap;
use home;
use sha256;

pub type Result<T = ()> = result::Result<T, String>;
const BUF_SIZE: usize = 1024;

fn get_default_kube_dir() -> Result<PathBuf> {
    let home_dir = match home::home_dir() {
        Some(path) => path.to_path_buf(),
        None => return Err(String::from("Cannot get home directory")),
    };

    let kube_dir = home_dir.join(".kube");
    return Ok(kube_dir);
}

fn get_default_kubenv_dir() -> Result<PathBuf> {
    let kube_dir = get_default_kube_dir()?;
    let kubenv_dir = kube_dir.join("kubenv");
    return Ok(kubenv_dir);
}

fn get_file_hash(path: &PathBuf) -> Result<String> {
    let hash = match sha256::try_digest(path.as_path()) {
        Ok(bytes) => bytes,
        Err(msg) => match path.to_str() {
            Some(ps) => {
                return Err(format!("Cannot get hash from file '{}': {}", ps, msg));
            }
            None => return Err(format!("Cannot get hash from file: {}", msg)),
        },
    };

    return Ok(hash);
}

#[derive(PartialEq)]
pub struct KubeConfig {
    name: String,
    path: PathBuf,
    hash: String,
}

impl KubeConfig {
    pub fn new(path: PathBuf, hash: String, name: Option<String>) -> Self {
        let name = match name {
            Some(n) => n,
            None => String::from(&hash[..8]),
        };

        return Self { name, path, hash };
    }

    pub fn name(&self) -> &str {
        return &self.name;
    }

    pub fn hash(&self) -> &str {
        return &self.hash;
    }
}

impl Clone for KubeConfig {
    fn clone(&self) -> Self {
        return Self {
            name: self.name.clone(),
            hash: self.hash.clone(),
            path: self.path.clone(),
        };
    }
}

impl PartialOrd for KubeConfig {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        return Some(self.name().cmp(other.name()));
    }
}

pub struct KubEnv {
    kube_dir: PathBuf,
    kubenv_dir: PathBuf,
    current_config: Option<KubeConfig>,
    configs: Vec<KubeConfig>,
    configs_by_name: HashMap<String, KubeConfig>,
    configs_by_hash: HashMap<String, KubeConfig>,
}

impl KubEnv {
    pub fn new(kubenv_dir: Option<PathBuf>, kube_dir: Option<PathBuf>) -> Result<Self> {
        let kube_dir = match kube_dir {
            Some(path_buf) => path_buf,
            None => get_default_kube_dir()?,
        };
        let kubenv_dir = match kubenv_dir {
            Some(path_buf) => path_buf,
            None => get_default_kubenv_dir()?,
        };
        let kubenv = Self {
            kube_dir,
            kubenv_dir,
            current_config: None,
            configs: vec![],
            configs_by_name: HashMap::new(),
            configs_by_hash: HashMap::new(),
        };
        return Ok(kubenv);
    }

    pub fn current_config(&self) -> Option<&KubeConfig> {
        return self.current_config.as_ref();
    }

    pub fn configs(&self) -> Vec<&KubeConfig> {
        return self.configs.iter().collect();
    }

    pub fn get_content(&self, name: &str) -> Result<BufReader<fs::File>> {
        let kubeconfig = match self.get_config_by_name(name) {
            Some(kc) => kc,
            None => return Err(format!("Cannot find config with name '{}'", name)),
        };

        return match fs::File::open(&kubeconfig.path) {
            Ok(f) => Ok(BufReader::with_capacity(BUF_SIZE, f)),
            Err(msg) => match kubeconfig.path.to_str() {
                Some(path) => Err(format!("Cannot open file '{}': {}", path, msg)),
                None => Err(format!("Cannot open file: {}", msg)),
            },
        };
    }

    pub fn set_content<R: Read>(&self, name: Option<String>, reader: &mut BufReader<R>) -> Result {
        let mut content: Vec<u8> = Vec::new();
        if let Err(msg) = reader.read_to_end(&mut content) {
            return Err(format!("Cannot read content from file: {}", msg));
        };

        let hash = sha256::digest(&content as &[u8]);
        if let Some(kc) = self.get_config_by_hash(&hash) {
            return Err(format!("Config already exists with name: '{}'", &kc.name));
        }
        let name = match name {
            Some(n) => {
                if let Some(kc) = self.get_config_by_name(&n) {
                    return Err(format!("Config with name '{}' already exists", kc.name));
                }
                n
            }
            None => hash.clone(),
        };

        let mut kubeconfig_filename = name.clone();
        kubeconfig_filename.push_str(".kubeconfig");
        let kubeconfig_path = self.kubenv_dir.join(kubeconfig_filename);
        let mut writer = match fs::File::create(&kubeconfig_path) {
            Ok(f) => BufWriter::with_capacity(BUF_SIZE, f),
            Err(msg) => match kubeconfig_path.to_str() {
                Some(p) => return Err(format!("Cannot open file '{}': {}", p, msg)),
                None => return Err(format!("Cannot open file: {}", msg)),
            },
        };
        if let Err(msg) = writer.write_all(&content) {
            match kubeconfig_path.to_str() {
                Some(p) => return Err(format!("Cannot write file '{}': {}", p, msg)),
                None => return Err(format!("Cannot write file: {}", msg)),
            }
        }

        return Ok(());
    }

    pub fn apply(&self, name: &str) -> Result {
        let kubeconfig = match self.get_config_by_name(name) {
            Some(kc) => kc,
            None => return Err(format!("Cannot find config with name '{}'", name)),
        };

        let config_file = self.kube_dir.join("config");
        if let Ok(hash) = get_file_hash(&config_file) {
            if hash == kubeconfig.hash {
                return Err(format!("Config '{}' already applied", name));
            }
        }
        if let Err(msg) = fs::copy(&kubeconfig.path, config_file) {
            return Err(format!(
                "Cannot copy config '{}' to config file: {}",
                kubeconfig.name, msg,
            ));
        }

        return Ok(());
    }

    pub fn remove(&self, name: &str) -> Result {
        let kubeconfig = match self.get_config_by_name(name) {
            Some(kc) => kc,
            None => return Err(format!("Cannot find config with name '{}'", name)),
        };

        if let Err(msg) = fs::remove_file(&kubeconfig.path) {
            return Err(format!(
                "Cannot remove config with name '{}': {}",
                kubeconfig.name, msg,
            ));
        }

        return Ok(());
    }

    pub fn sync(&mut self) -> Result {
        if !self.kubenv_dir.is_dir() {
            if let Err(msg) = fs::create_dir_all(self.kubenv_dir.as_path()) {
                match self.kubenv_dir.to_str() {
                    Some(path) => {
                        return Err(format!(
                            "Cannot create kubenv directory '{}': {}",
                            path, msg,
                        ))
                    }
                    None => return Err(format!("Cannot create kubenv directory: {}", msg)),
                }
            }
        };

        self.update_configs()?;
        _ = self.update_current_config();

        return Ok(());
    }

    fn get_config_by_name(&self, name: &str) -> Option<&KubeConfig> {
        return self.configs_by_name.get(name);
    }

    fn get_config_by_hash(&self, hash: &str) -> Option<&KubeConfig> {
        return self.configs_by_hash.get(hash);
    }

    fn update_configs(&mut self) -> Result {
        let config_files = match fs::read_dir(&self.kubenv_dir) {
            Ok(value) => value,
            Err(msg) => match self.kubenv_dir.to_str() {
                Some(path) => {
                    return Err(format!(
                        "Cannot read files from directory '{}': {}",
                        path, msg,
                    ))
                }
                None => return Err(format!("Cannot read files from kubenv directory: {}", msg)),
            },
        };

        self.configs.clear();
        self.configs_by_name.clear();
        self.configs_by_hash.clear();
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
                let hash = match get_file_hash(&path) {
                    Ok(h) => h,
                    Err(_) => continue,
                };
                if let Err(_) = self.add(KubeConfig::new(path, hash, Some(name))) {
                    continue;
                };
            };
        }

        return Ok(());
    }

    fn update_current_config(&mut self) -> Result {
        let current_config_file = self.kube_dir.join("config");
        let hash = get_file_hash(&current_config_file)?;

        if current_config_file.is_file() {
            let kubeconfig = KubeConfig::new(current_config_file, hash, None);
            self.current_config = Some(kubeconfig.clone());
            _ = self.add(kubeconfig.clone());
        }

        return Ok(());
    }

    fn add(&mut self, kubeconfig: KubeConfig) -> Result {
        if let Some(kc) = self.get_config_by_name(&kubeconfig.name) {
            return Err(format!("Config with name '{}' already exists", kc.name));
        }
        if let Some(kc) = self.get_config_by_hash(&kubeconfig.hash) {
            return Err(format!("Config already exists with name '{}'", kc.name));
        }

        // Add to self.configs
        let kubeconfig_tmp = kubeconfig.clone();
        let len = self.configs.len();
        let mut index = 0;
        if len > 0 {
            index = len - 1;
            while index > 0 && kubeconfig_tmp < self.configs[index - 1] {
                index -= 1;
            }
        }
        self.configs.insert(index, kubeconfig_tmp);

        // Add to self.configs_by_name
        let kubeconfig_tmp = kubeconfig.clone();
        self.configs_by_name
            .insert(kubeconfig_tmp.name.clone(), kubeconfig_tmp);

        // Add to self.configs_by_hash
        let kubeconfig_tmp = kubeconfig.clone();
        self.configs_by_hash
            .insert(kubeconfig_tmp.hash.clone(), kubeconfig_tmp);

        return Ok(());
    }
}
