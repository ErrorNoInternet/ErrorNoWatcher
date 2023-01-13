# ErrorNoWatcher
ErrorNoWatcher is a Minecraft bot (written in Rust with [azalea](https://github.com/mat-1/azalea)) that alerts you when players are near your base. It also has other features such as interacting with blocks/entities, a basic pathfinder that can follow players and go to coordinates, a scripting system to run commands automatically, and much more.

## Compiling
```sh
git clone https://github.com/ErrorNoInternet/ErrorNoWatcher
cd ErrorNoWatcher
cargo build --release
```
The compiled executable will be at `./target/release/errornowatcher`

## Usage
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
	"-HTitle: Intruder Alert",
	"-HPriority: urgent",
	"-HTags: warning",
	"-d{player_name} is near your base! Their coordinates are {x} {y} {z}.",
	"<your URL here (or a service such as ntfy.sh)>",
]
```
Run `/msg <bot username> help` to see a list of commands you can run.\
For example, `BotStatus` = `/msg <bot username> bot_status`
