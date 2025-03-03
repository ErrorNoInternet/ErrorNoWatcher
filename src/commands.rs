use crate::{
    State,
    lua::{eval, exec, reload},
};
use azalea::{brigadier::prelude::*, chat::ChatPacket, prelude::*};
use futures::lock::Mutex;

pub type Ctx = CommandContext<Mutex<CommandSource>>;

pub struct CommandSource {
    pub client: Client,
    pub message: ChatPacket,
    pub state: State,
}

impl CommandSource {
    pub fn reply(&self, message: &str) {
        for chunk in message
            .chars()
            .collect::<Vec<char>>()
            .chunks(236)
            .map(|chars| chars.iter().collect::<String>())
        {
            self.client.chat(
                &(if self.message.is_whisper()
                    && let Some(username) = self.message.username()
                {
                    format!("/w {username} {chunk}")
                } else {
                    chunk
                }),
            );
        }
    }
}

pub fn register(commands: &mut CommandDispatcher<Mutex<CommandSource>>) {
    commands.register(literal("reload").executes(|ctx: &Ctx| {
        let source = ctx.source.clone();
        tokio::spawn(async move {
            let source = source.lock().await;
            source.reply(&format!(
                "{:?}",
                reload(&source.state.lua, source.message.username())
            ));
        });
        1
    }));

    commands.register(
        literal("eval").then(argument("code", string()).executes(|ctx: &Ctx| {
            let source = ctx.source.clone();
            let code = get_string(ctx, "code").expect("argument should exist");
            tokio::spawn(async move {
                let source = source.lock().await;
                source.reply(&format!(
                    "{:?}",
                    eval(&source.state.lua, &code, source.message.username()).await
                ));
            });
            1
        })),
    );

    commands.register(
        literal("exec").then(argument("code", string()).executes(|ctx: &Ctx| {
            let source = ctx.source.clone();
            let code = get_string(ctx, "code").expect("argument should exist");
            tokio::spawn(async move {
                let source = source.lock().await;
                source.reply(&format!(
                    "{:?}",
                    exec(&source.state.lua, &code, source.message.username()).await
                ));
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
