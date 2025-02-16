use crate::{
    State,
    scripting::{eval, exec, reload},
};
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
        source.reply(&format!("{:?}", reload(&source.state.lua.lock())));
        1
    }));

    commands.register(
        literal("eval").then(argument("code", string()).executes(|ctx: &Ctx| {
            let source = ctx.source.lock();
            source.reply(&format!(
                "{:?}",
                eval(&source.state.lua.lock(), &get_string(ctx, "code").unwrap())
            ));
            1
        })),
    );

    commands.register(
        literal("exec").then(argument("code", string()).executes(|ctx: &Ctx| {
            let source = ctx.source.lock();
            source.reply(&format!(
                "{:?}",
                exec(&source.state.lua.lock(), &get_string(ctx, "code").unwrap())
            ));
            1
        })),
    );

    commands.register(literal("ping").executes(|ctx: &Ctx| {
        ctx.source.lock().reply("pong!");
        1
    }));
}
