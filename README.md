# KubeMan

CLI application for managing kubernetes configs.

## Installation

## Quickstart

See configs list
```
kubeman list
```

Apply any config
```
kubeman apply config_name
```

Add new config
```
kubeman add --name config_name --file /config/path
```
or
```
cat /config/path | kubeman add --name config_name
```

Export config
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

