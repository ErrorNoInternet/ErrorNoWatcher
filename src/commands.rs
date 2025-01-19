use crate::State;
use azalea::{
    GameProfileComponent, brigadier::prelude::*, chat::ChatPacket, entity::metadata::Player,
    prelude::*,
};
use bevy_ecs::{entity::Entity, query::With};
use parking_lot::Mutex;

pub type Ctx<'a> = CommandContext<Mutex<CommandSource>>;

pub struct CommandSource {
    pub client: Client,
    pub message: ChatPacket,
    pub state: State,
}

impl CommandSource {
    pub fn reply(&self, message: &str) {
        if self.message.is_whisper()
            && let Some(username) = self.message.username()
        {
            self.client.chat(&format!("/w {username} {message}"));
        } else {
            self.client.chat(message);
        }
    }

    pub fn _entity(&mut self) -> Option<Entity> {
        let username = self.message.username()?;
        self.client
            .entity_by::<With<Player>, &GameProfileComponent>(|profile: &&GameProfileComponent| {
                profile.name == username
            })
    }
}

pub fn register(commands: &mut CommandDispatcher<Mutex<CommandSource>>) {
    commands.register(literal("reload").executes(|ctx: &Ctx| {
        let source = ctx.source.lock();
        let lua = source.state.lua.lock();
        let config_path = match lua.globals().get::<String>("config_path") {
            Ok(path) => path,
            Err(error) => {
                source.reply(&format!(
                    "failed to get config_path from lua globals: {error:?}"
                ));
                return 0;
            }
        };
        if let Err(error) = match &std::fs::read_to_string(&config_path) {
            Ok(string) => lua.load(string).exec(),
            Err(error) => {
                source.reply(&format!("failed to read {config_path:?}: {error:?}"));
                return 0;
            }
        } {
            source.reply(&format!(
                "failed to execute configuration as lua code: {error:?}"
            ));
            return 0;
        }
        1
    }));

    commands.register(
        literal("eval").then(argument("expr", string()).executes(|ctx: &Ctx| {
            let source = ctx.source.lock();
            source.reply(&format!(
                "{:?}",
                source
                    .state
                    .lua
                    .lock()
                    .load(get_string(ctx, "expr").unwrap())
                    .eval::<String>()
            ));
            1
        })),
    );

    commands.register(
        literal("exec").then(argument("code", string()).executes(|ctx: &Ctx| {
            let source = ctx.source.lock();
            source.reply(&format!(
                "{:?}",
                source
                    .state
                    .lua
                    .lock()
                    .load(get_string(ctx, "code").unwrap())
                    .exec()
            ));
            1
        })),
    );

    commands.register(literal("ping").executes(|ctx: &Ctx| {
        ctx.source.lock().reply("pong!");
        1
    }));
}
