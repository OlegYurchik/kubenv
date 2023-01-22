# KubEnv

CLI application for managing kubernetes environments.

## Installation

### Build
```bash
git clone https://github.com/OlegYurchik/kubenv
cd kubenv
cargo build --release
sudo cp ./target/release/kubenv /usr/local/bin/kubenv
```

### Binary
```bash
wget -c https://github.com/OlegYurchik/kubenv/releases/latest/download/kubenv.tar.gz -O - | tar -xz
sudo mv ./kubenv /usr/local/bin/kubenv
```

## Kubectl Plugin

After installation you can setup KubEnv like a Kubectl plugin
```bash
sudo ln -sf /usr/local/bin/kubectl-env /usr/local/bin/kubenv
```

And run KubEnv as `kubectl env`. Example:
```bash
kubectl env list
```

## Quickstart

### Configs list
```bash
kubenv list
```

### Apply config
```bash
kubenv apply config_name
```

### Add config
```bash
kubenv add --name config_name --file /config/path
```
or
```bash
kubenv add --name config_name < /config/path
```
or
```bash
cat /config/path | kubenv add --name config_name
```

### Export config
```bash
kubenv export config_name --file /new/config/path
```
or
```bash
kubenv export config_name > /new/config/path
```

## TODO

1. Now commands `export` and `import` working with full content. Need change to working through
buffer. Need remove `export` method from `KubEnv` struct implementation
 
