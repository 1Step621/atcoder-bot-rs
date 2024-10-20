use std::{
    collections::{HashMap, HashSet},
    fs,
    sync::Mutex,
    time::Duration,
};

use ::serenity::all::CacheHttp;
use anyhow::Error;
use chrono::{Local, Timelike};
use dotenvy::dotenv;
use poise::serenity_prelude as serenity;
use reqwest::{
    header::{HeaderMap, ACCEPT_ENCODING},
    Client,
};
use serde::{Deserialize, Serialize};
use serenity::all::{CreateEmbed, CreateMessage, Mentionable};
use tokio::time::{sleep_until, Instant};

#[derive(Serialize, Deserialize, Debug, Default)]
struct Data {
    channel: Mutex<Option<serenity::ChannelId>>,
    users: Mutex<HashSet<String>>,
}
type Context<'a> = poise::Context<'a, Data, Error>;

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

#[derive(PartialEq, PartialOrd, Eq, Ord)]
enum Color {
    BLACK, // for unknown difficulty
    GRAY,
    BROWN,
    GREEN,
    CYAN,
    BLUE,
    YELLOW,
    ORANGE,
    RED,
}

impl From<Color> for u32 {
    fn from(val: Color) -> Self {
        match val {
            Color::BLACK => 0x000000,
            Color::GRAY => 0x808080,
            Color::BROWN => 0xa52a2a,
            Color::GREEN => 0x008000,
            Color::CYAN => 0x00ffff,
            Color::BLUE => 0x0000ff,
            Color::YELLOW => 0xffff00,
            Color::ORANGE => 0xffa500,
            Color::RED => 0xff0000,
        }
    }
}

impl From<Color> for String {
    fn from(val: Color) -> Self {
        match val {
            Color::BLACK => unreachable!(),
            Color::GRAY => "灰".to_string(),
            Color::BROWN => "茶".to_string(),
            Color::GREEN => "緑".to_string(),
            Color::CYAN => "水".to_string(),
            Color::BLUE => "青".to_string(),
            Color::YELLOW => "黄".to_string(),
            Color::ORANGE => "橙".to_string(),
            Color::RED => "赤".to_string(),
        }
    }
}

fn normalize_difficulty(difficulty: i64) -> i64 {
    if difficulty >= 400 {
        difficulty
    } else {
        (400.0 / (1.0 + (1.0 - difficulty as f64 / 400.0).exp())) as i64
    }
}

fn difficulty_color(difficulty: i64) -> Color {
    match difficulty {
        0..=399 => Color::GRAY,
        400..=799 => Color::BROWN,
        800..=1199 => Color::GREEN,
        1200..=1599 => Color::CYAN,
        1600..=1999 => Color::BLUE,
        2000..=2399 => Color::YELLOW,
        2400..=2799 => Color::ORANGE,
        _ => Color::RED,
    }
}

/// メッセージを送信するチャンネルを設定します。
#[poise::command(slash_command)]
async fn channel(ctx: Context<'_>) -> Result<(), Error> {
    *ctx.data().channel.lock().unwrap() = Some(ctx.channel_id());
    save(ctx.data())?;
    ctx.reply(format!(
        "チャンネルを {} に設定しました。",
        ctx.channel_id().mention()
    ))
    .await?;
    Ok(())
}

/// AtCoderのユーザーを登録します。カンマ区切りで複数人指定できます。
#[poise::command(slash_command)]
async fn register(
    ctx: Context<'_>,
    #[description = "AtCoderのユーザー名"] users: String,
) -> Result<(), Error> {
    let users = users
        .split(",")
        .map(|u| u.trim().to_string())
        .collect::<Vec<_>>();
    ctx.reply(format!("ユーザー ({}) を登録しました。", users.join(", ")))
        .await?;
    println!("User registered: {:?}", &users);
    ctx.data().users.lock().unwrap().extend(users);
    save(ctx.data())?;
    Ok(())
}

/// AtCoderのユーザーを登録解除します。
#[poise::command(slash_command)]
async fn unregister(
    ctx: Context<'_>,
    #[description = "AtCoderのユーザー名"] user: String,
) -> Result<(), Error> {
    ctx.data().users.lock().unwrap().remove(&user);
    save(ctx.data())?;
    ctx.reply(format!("ユーザー ({}) を登録解除しました。", user))
        .await?;
    println!("User unregistered: {:?}", &user);
    Ok(())
}

/// 登録されているユーザーの一覧を表示します。
#[poise::command(slash_command)]
async fn registerlist(ctx: Context<'_>) -> Result<(), Error> {
    let mut users = ctx
        .data()
        .users
        .lock()
        .unwrap()
        .iter()
        .cloned()
        .collect::<Vec<_>>();
    users.sort();
    let users = users;
    ctx.reply(format!("登録されているユーザー: {}", users.join(", ")))
        .await?;
    Ok(())
}

async fn process(
    channel: serenity::ChannelId,
    users: HashSet<String>,
    cache: impl CacheHttp,
) -> Result<(), Error> {
    #[allow(unused)]
    #[derive(Clone, Deserialize, Debug, Default)]
    struct ProblemModelItem {
        slope: Option<f64>,
        intercept: Option<f64>,
        variance: Option<f64>,
        difficulty: Option<i64>,
        discrimination: Option<f64>,
        irt_loglikelihood: Option<f64>,
        irt_users: Option<i64>,
        is_experimental: Option<bool>,
    }

    #[allow(unused)]
    #[derive(Clone, Deserialize, Debug, Default)]
    struct ProblemItem {
        id: String,
        contest_id: String,
        problem_index: String,
        name: String,
        title: String,
    }

    #[allow(unused)]
    #[derive(Deserialize, Debug)]
    struct SubmissionItem {
        id: i64,
        epoch_second: i64,
        problem_id: String,
        contest_id: String,
        user_id: String,
        language: String,
        point: f32,
        length: i32,
        result: String,
        execution_time: i32,
    }

    struct ProblemDetail {
        title: String,
        difficulty: Option<i64>,
        language: String,
        submission_url: String,
    }

    impl ProblemDetail {
        fn to_field(&self) -> (String, String, bool) {
            (
                self.title.clone(),
                format!(
                    "{} | {} | [提出]({})",
                    self.difficulty
                        .map(|d| {
                            let diff = normalize_difficulty(d);
                            format!("{}({})", diff, Into::<String>::into(difficulty_color(diff)))
                        })
                        .unwrap_or("不明".into()),
                    self.language,
                    self.submission_url
                ),
                false,
            )
        }
    }

    let client = Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT_ENCODING, "gzip".parse().unwrap());

    let res = client
        .get("https://kenkoooo.com/atcoder/resources/problem-models.json")
        .headers(headers.clone())
        .send()
        .await?
        .error_for_status()?;
    // let problem_models = res.json::<HashMap<String, ProblemModelItem>>().await?;
    let problem_models =
        serde_json::from_str::<HashMap<String, ProblemModelItem>>(&res.text().await?)?;
    let res = client
        .get("https://kenkoooo.com/atcoder/resources/problems.json")
        .headers(headers.clone())
        .send()
        .await?
        .error_for_status()?;
    // let problems = res.json::<Vec<ProblemItem>>().await?;
    let problems = serde_json::from_str::<Vec<ProblemItem>>(&res.text().await?)?;

    let mut embeds = vec![];
    for user in users {
        let res = client
            .get(format!(
                "https://kenkoooo.com/atcoder/atcoder-api/v3/user/submissions?user={}&from_second={}",
                user, Local::now().timestamp() - 24 * 60 * 60
            ))
            .headers(headers.clone())
            .send()
            .await?
            .error_for_status()?;
        // let submissions = res.json::<Vec<SubmissionItem>>().await?;
        let submissions = serde_json::from_str::<Vec<SubmissionItem>>(&res.text().await?)?;

        let solved_ids = submissions
            .iter()
            .filter(|s| s.result == "AC")
            .map(|s| s.problem_id.clone())
            .collect::<Vec<_>>();

        let solved_problems = solved_ids
            .iter()
            .map(|id| {
                let problem_model = problem_models.get(id).cloned().unwrap_or_default();
                let problem = problems
                    .iter()
                    .find(|p| p.id == *id)
                    .cloned()
                    .unwrap_or_default();
                let submission = submissions.iter().find(|s| s.problem_id == *id).unwrap();
                ProblemDetail {
                    title: problem.title.clone(),
                    difficulty: problem_model.difficulty,
                    language: submission.language.clone(),
                    submission_url: format!(
                        "https://atcoder.jp/contests/{}/submissions?f.User={}",
                        problem.contest_id, user
                    ),
                }
            })
            .collect::<Vec<_>>();

        embeds.extend(solved_problems.windows(25).map(|problems| {
            CreateEmbed::default()
                .fields(problems.iter().map(|p| p.to_field()))
                .color(Into::<u32>::into(
                    problems
                        .iter()
                        .map(|p| {
                            p.difficulty
                                .map(normalize_difficulty)
                                .map(difficulty_color)
                                .unwrap_or(Color::BLACK)
                        })
                        .max()
                        .unwrap(),
                ))
        }));
    }

    if embeds.is_empty() {
        channel
            .send_message(
                cache,
                CreateMessage::default().content("昨日は誰もACしませんでした。"),
            )
            .await?;
    } else {
        channel
            .send_message(cache, CreateMessage::default().embeds(embeds))
            .await?;
    }

    Ok(())
}

#[poise::command(slash_command)]
async fn run(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;
    let channel = ctx
        .data()
        .channel
        .lock()
        .unwrap()
        .clone()
        .expect("Channel not set");
    let users = ctx.data().users.lock().unwrap().clone();
    process(channel, users, ctx.http()).await?;
    ctx.reply("完了！").await?;
    Ok(())
}

async fn daily_job(channel: serenity::ChannelId, users: HashSet<String>, cache: impl CacheHttp) {
    loop {
        let now = Local::now();
        let target_time = (Local::now() + chrono::Duration::days(1))
            .with_hour(0)
            .and_then(|d| d.with_minute(0))
            .and_then(|d| d.with_second(0))
            .unwrap();
        let sleep_duration = Duration::from_secs((target_time.timestamp() - now.timestamp()).try_into().unwrap());

        println!("Now: {}", now);
        println!("Next run: {}", target_time);
        println!("Sleeping for {} seconds", sleep_duration.as_secs());

        sleep_until(Instant::now() + sleep_duration).await;
        process(channel, users.clone(), cache.http()).await.expect("Failed to run daily job");
    }
}

async fn event_handler(
    ctx: serenity::Context,
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
        tokio::spawn(daily_job(
            data.channel.lock().unwrap().clone().expect(""),
            data.users.lock().unwrap().clone(),
            ctx.http.clone(),
        ));
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv().expect(".env file not found");

    let token = std::env::var("DISCORD_TOKEN").expect("Missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![channel(), register(), unregister(), registerlist(), run()],
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx.clone(), event, framework, data))
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
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
