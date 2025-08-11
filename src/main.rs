use std::{collections::BTreeSet, fmt::Display, fs, sync::Mutex};

use anyhow::Error;
use dotenvy::dotenv;
use poise::{ChoiceParameter, serenity_prelude as serenity};
use serde::{Deserialize, Serialize};

mod api_parsing;
mod functions;

type PoiseContext<'a> = poise::Context<'a, Data, Error>;

#[derive(
    Clone, Copy, Serialize, Deserialize, Debug, PartialEq, PartialOrd, Eq, Ord, ChoiceParameter,
)]
enum WellKnownContest {
    #[name = "ABC"]
    Abc,
    #[name = "ARC"]
    Arc,
    #[name = "AGC"]
    Agc,
    #[name = "AHC"]
    Ahc,
}

impl Display for WellKnownContest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Abc => "ABC",
                Self::Arc => "ARC",
                Self::Agc => "AGC",
                Self::Ahc => "AHC",
            }
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct Data {
    users: Mutex<BTreeSet<String>>,
    submissions_channel: Mutex<Option<serenity::ChannelId>>,
    mention: Mutex<Option<serenity::RoleId>>,
    contest_kind: Mutex<BTreeSet<WellKnownContest>>,
    contests_channel: Mutex<Option<serenity::ChannelId>>,
}

fn save(data: &Data) -> Result<(), Error> {
    let data = serde_json::to_string(data)?;
    std::fs::write("config.json", data)?;
    Ok(())
}

fn load() -> Result<Data, Error> {
    let data = fs::read_to_string("config.json")?;
    let data = serde_json::from_str(&data).expect("Failed to parse config.json");
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
                *data.submissions_channel.lock().unwrap() =
                    *restore.submissions_channel.lock().unwrap();
                *data.users.lock().unwrap() = restore.users.lock().unwrap().clone();
                *data.contest_kind.lock().unwrap() = restore.contest_kind.lock().unwrap().clone();
                *data.mention.lock().unwrap() = *restore.mention.lock().unwrap();
                *data.contests_channel.lock().unwrap() = *restore.contests_channel.lock().unwrap();
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
                commands::set_submissions_channel(),
                commands::set_contests_channel(),
                commands::register(),
                commands::unregister(),
                commands::register_list(),
                commands::run(),
                commands::enable_contest_notification(),
                commands::disable_contest_notification(),
                commands::set_mention(),
            ],
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                periodic::start_waiting(ctx.clone());
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
