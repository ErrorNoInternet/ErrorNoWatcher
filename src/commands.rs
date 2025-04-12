use azalea::{brigadier::prelude::*, chat::ChatPacket, prelude::*};
use futures::lock::Mutex;
use mlua::{Function, Table};
use ncr::utils::prepend_header;

use crate::{
    State,
    lua::{eval, exec, reload},
};

pub type Ctx = CommandContext<Mutex<CommandSource>>;

pub struct CommandSource {
    pub client: Client,
    pub message: ChatPacket,
    pub state: State,
    pub ncr_options: Option<Table>,
}

impl CommandSource {
    pub fn reply(&self, message: &str) {
        for mut chunk in message
            .chars()
            .collect::<Vec<char>>()
            .chunks(if self.ncr_options.is_some() { 150 } else { 236 })
            .map(|chars| chars.iter().collect::<String>())
        {
            if let Some(options) = &self.ncr_options
                && let Ok(encrypt) = self.state.lua.globals().get::<Function>("ncr_encrypt")
                && let Ok(ciphertext) = encrypt.call::<String>((options, prepend_header(&chunk)))
            {
                chunk = ciphertext;
            }
            self.client.chat(
                &(if self.message.is_whisper()
                    && let Some(username) = self.message.sender()
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
            source.reply(
                &reload(&source.state.lua, source.message.sender())
                    .map_or_else(|error| error.to_string(), |()| String::from("ok")),
            );
        });
        1
    }));

    commands.register(
        literal("eval").then(argument("code", string()).executes(|ctx: &Ctx| {
            let source = ctx.source.clone();
            let code = get_string(ctx, "code").expect("argument should exist");
            tokio::spawn(async move {
                let source = source.lock().await;
                source.reply(
                    &eval(&source.state.lua, &code, source.message.sender())
                        .await
                        .unwrap_or_else(|error| error.to_string()),
                );
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
                source.reply(
                    &exec(&source.state.lua, &code, source.message.sender())
                        .await
                        .map_or_else(|error| error.to_string(), |()| String::from("ok")),
                );
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
