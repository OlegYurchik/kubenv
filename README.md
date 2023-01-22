# KubeMan

CLI application for managing kubernetes configs.

## Installation

### Build

```
git clone https://github.com/OlegYurchik/kubeman
cd kubeman
cargo build --release
sudo cp ./target/release/kubeman /usr/local/bin/kubeman
```

### Binary

```
wget -c https://github.com/OlegYurchik/kubeman/releases/latest/download/kubeman.tar.gz -O - | tar -xz
sudo mv ./kubeman /usr/local/bin/kubeman
```

## Quickstart

### Configs list
```
kubeman list
```

### Apply config
```
kubeman apply config_name
```

### Add config
```
kubeman add --name config_name --file /config/path
```
or
```
cat /config/path | kubeman add --name config_name
```

### Export config
```
kubeman export config_name --file /new/config/path
```
or
```
kubeman export config_name > /new/config/path
```

## TODO

1. Now commands `export` and `import` working with full content. Need change to working through
buffer.

