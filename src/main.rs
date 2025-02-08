use std::{collections::BTreeSet, fs, sync::Mutex};

use anyhow::Error;
use dotenvy::dotenv;
use poise::serenity_prelude as serenity;
use serde::{Deserialize, Serialize};

mod functions;
mod api_parsing;
mod notify;

type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(Serialize, Deserialize, Debug, Default)]
struct Data {
    channel: Mutex<Option<serenity::ChannelId>>,
    users: Mutex<BTreeSet<String>>,
}

fn save(data: &Data) -> Result<(), Error> {
    let data = serde_json::to_string(data)?;
    std::fs::write("config.json", data)?;
    Ok(())
}

fn load() -> Result<Data, Error> {
    let data = fs::read_to_string("config.json")?;
    let data = serde_json::from_str(&data)?;
    Ok(data)
}

async fn event_handler(
    _ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    if let serenity::FullEvent::Ready { data_about_bot } = event {
        println!("Logged in as {}", data_about_bot.user.name);
        match load() {
            Ok(restore) => {
                *data.channel.lock().unwrap() = *restore.channel.lock().unwrap();
                *data.users.lock().unwrap() = restore.users.lock().unwrap().clone();
                println!("Config restored:");
                println!("{:#?}", data);
            }
            Err(_) => {
                println!("Note: config.json not found, using default data");
            }
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    use functions::*;
    
    dotenv().expect(".env file not found");

    let token = std::env::var("DISCORD_TOKEN").expect("Missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::channel(),
                commands::register(),
                commands::unregister(),
                commands::registerlist(),
                commands::run(),
            ],
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                tokio::spawn(periodic::wait(ctx.clone()));
                Ok(Data::default())
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client
        .expect("Failed to create client")
        .start()
        .await
        .expect("Failed to start client");
}
