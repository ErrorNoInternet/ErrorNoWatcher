# ErrorNoWatcher
ErrorNoWatcher is a Minecraft bot (written in Rust with [azalea](https://github.com/mat-1/azalea)) that alerts you when players are near your base. It also has other features such as interacting with blocks/entities, a basic pathfinder that can follow players and go to coordinates, a scripting system to run commands automatically, and much more.

## Compiling
```sh
git clone https://github.com/ErrorNoInternet/ErrorNoWatcher
cd ErrorNoWatcher
cargo build --release
```
The compiled executable can be found at `./target/release/errornowatcher`

## Usage
### Configuration
Running the bot will create the `bot_configuration.toml` file, where you can change several options:
```toml
username = "<bot's username>"
server_address = "<server address>"
register_keyword = "Register using"
register_command = "register MyPassword MyPassword"
login_keyword = "Login using"
login_command = "login MyPassword"
bot_owners = ["ErrorNoInternet", "<allowed to run commands>"]
whitelist = [
	"ErrorNoInternet",
	"<won't be triggered by the alert system>"
]
alert_players = ["ErrorNoInternet", "<players to send a message to>"]
alert_location = [0, 0]
alert_radius = 192
alert_command = [
	"curl",
	"-s",
	"-m 5",
	"-HTitle: Intruder Alert",
	"-HPriority: urgent",
	"-HTags: warning",
	"-d{player_name} is near your base! Their coordinates are {x} {y} {z}.",
	"<your URL here (or a service such as ntfy.sh)>",
]
```
### Example commands
- `/msg ErrorNoWatcher help 1` - list the first page of usable commands
- `/msg ErrorNoWatcher bot_status` - display the bot's health, food & saturation levels, etc
- `/msg ErrorNoWatcher goto 20 64 10` - go to X20 Y64 Z10 (using the pathfinder)
- `/msg ErrorNoWatcher script sleep.txt` - run all commands in the file `sleep.txt`
- `/msg ErrorNoWatcher attack ErrorNoInternet` - attack the player named ErrorNoInternet
- `/msg ErrorNoWatcher look 180 0` - rotate the bot's head to 180 (yaw) 0 (pitch)
- `/msg ErrorNoWatcher whitelist_add Notch` - temporarily add Notch to the whitelist
- `/msg ErrorNoWatcher sprint forward 5000` - sprint forward for 5 seconds
- `/msg ErrorNoWatcher drop_item` - drop the currently held item (or `drop_stack`)
- `/msg ErrorNoWatcher last_location 1` - show the first page of players sorted by join time
- `/msg ErrorNoWatcher last_location ErrorNoInternet` - display the last seen location
- `/msg ErrorNoWatcher follow_player ErrorNoInternet` - start following ErrorNoInternet
- `/msg ErrorNoWatcher slot 1` - switch to the first inventory slot (hold the item)
