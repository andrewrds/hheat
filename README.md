# hheat
Control UK Hive smart thermostat from the command line.

## Setup
To set username and password, create a simple TOML file `~/.hheat/conf.toml` in your user home directory with the credentials for your Hive heating account:
```toml
username = "username@example.com"
password = "passw0rd"
```
## Running
### Display Status
To display the current heating status, run `hheat` without any arguments:
```fish
➜ hheat
Mode          schedule
Temperature     21.3°
Target          22.0° 🔥
```

### Set target temperature
To set the target temperature, run `hheat <temp>`:
```fish
➜ hheat 20.5
```
This will change the mode to manual if it is currently off.

### Change mode
To change between schedule, manual and off modes, run `hheat schedule|manual|off`:
```fish
➜ hheat off
```

## How it works
This program calls the Hive smart thermostat web service API. This means it doesn't require a local connection to the Hive thermostat and will work from anywhere.

After initial login, the login token will be saved in `~/.hheat/token`. This token is then used for susequent calls until it expires to avoid repeated logon calls.
