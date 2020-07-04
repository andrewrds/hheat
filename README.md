# hive-heat
Control UK Hive smart thermostat from the command line.

## Setup
To set username and password, create a simple TOML file `~/.hive-heat/conf.toml` in your user home directory with the credentials for your Hive heating account:
```toml
username = "username@example.com"
password = "passw0rd"
```
## Running
### Display Status
To display the current heating status, run `hive-heat` without any arguments:
```fish
➜ hive-heat
Temperature   20.1°C
Target         7.0°C
```

### Set target temperature
To set the target temperature, run `hive-heat <temp>`:
```fish
➜ hive-heat 20.5
```

## How it works
This program calls the Hive smart thermostat web service API. This means it doesn't require a local connection to the Hive thermostat and will work from anywhere.
