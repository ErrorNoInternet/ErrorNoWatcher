use crate::{
    State,
    scripting::{eval, exec, reload},
};
use azalea::{
    GameProfileComponent, brigadier::prelude::*, chat::ChatPacket, entity::metadata::Player,
    prelude::*,
};
use bevy_ecs::{entity::Entity, query::With};
use futures::lock::Mutex;

pub type Ctx<'a> = CommandContext<Mutex<CommandSource>>;

pub struct CommandSource {
    pub client: Client,
    pub message: ChatPacket,
    pub state: State,
}

impl CommandSource {
    pub fn reply(&self, message: &str) {
        let response = if self.message.is_whisper()
            && let Some(username) = self.message.username()
        {
            &format!("/w {username} {message}")
        } else {
            message
        };
        self.client.chat(response);
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
        let source = ctx.source.clone();
        tokio::spawn(async move {
            let source = source.lock().await;
            source.reply(&format!("{:?}", reload(&source.state.lua)));
        });
        1
    }));

    commands.register(
        literal("eval").then(argument("code", string()).executes(|ctx: &Ctx| {
            let source = ctx.source.clone();
            let code = get_string(ctx, "code").unwrap();
            tokio::spawn(async move {
                let source = source.lock().await;
                source.reply(&format!("{:?}", eval(&source.state.lua, &code).await));
            });
            1
        })),
    );

    commands.register(
        literal("exec").then(argument("code", string()).executes(|ctx: &Ctx| {
            let source = ctx.source.clone();
            let code = get_string(ctx, "code").unwrap();
            tokio::spawn(async move {
                let source = source.lock().await;
                source.reply(&format!("{:?}", exec(&source.state.lua, &code).await));
            });
            1
        })),
    );

    commands.register(literal("ping").executes(|ctx: &Ctx| {
        let source = ctx.source.clone();
        tokio::spawn(async move {
            source.lock().await.reply("pong!");
        });
        1
    }));
}
