# ErrorNoWatcher

A Minecraft bot with Lua scripting support, written in Rust with [azalea](https://github.com/azalea-rs/azalea).

## Features

- Running Lua from
    - `errornowatcher.lua`
    - in-game chat messages
    - POST requests to HTTP server
- Listening to in-game events
- Pathfinding (from azalea)
- Entity and chest interaction
- NoChatReports encryption

## Usage

```sh
$ git clone https://github.com/ErrorNoInternet/ErrorNoWatcher
$ cd ErrorNoWatcher
$ cargo build --release
$ # ./target/release/errornowatcher
```

Make sure the `Server` and `Username` globals are defined in `errornowatcher.lua` before starting the bot.
