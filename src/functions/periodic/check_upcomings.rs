use std::sync::Arc;

use anyhow::{Context as _, Error};
use chrono::{Duration, NaiveTime, Utc};
use poise::serenity_prelude::{self as serenity, Color, CreateEmbed, CreateMessage, Mentionable};
use tokio::time::{Instant, sleep_until};

use crate::{WellKnownContest, api_parsing::types::ContestItem, load, save};

pub async fn check_upcomings(ctx: &serenity::Context) -> Result<(), Error> {
    println!("Checking upcoming contests...");

    let data = load()?;
    let contest_notification = data.contest_kind.lock().unwrap().clone();

    let contests = reqwest::get("https://atcoder-upcoming-contests-cs7x.shuttle.app/")
        .await?
        .error_for_status()?
        .text()
        .await?;
    let contests: Vec<ContestItem> = serde_json::from_str(&contests)?;

    let next_run = (Utc::now() + Duration::days(1))
        .with_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
        .unwrap();

    const NOTIFICATION_BEFORE: Duration = Duration::minutes(5);

    let contests = contests
        .into_iter()
        .filter(|contest| {
            contest_notification
                .iter()
                .map(|&c| match c {
                    WellKnownContest::Abc => |s: &String| s.contains("AtCoder Beginner Contest"),
                    WellKnownContest::Arc => |s: &String| s.contains("AtCoder Regular Contest"),
                    WellKnownContest::Agc => |s: &String| s.contains("AtCoder Grand Contest"),
                    WellKnownContest::Ahc => |s: &String| s.contains("AtCoder Heuristic Contest"),
                })
                .any(|f| f(&contest.name))
        })
        .filter(|contest| {
            Utc::now() <= contest.start_time - NOTIFICATION_BEFORE
                && contest.start_time - NOTIFICATION_BEFORE < next_run
        })
        .collect::<Vec<_>>();

    let ctx = Arc::new(ctx.clone());

    for contest in contests {
        let ctx = ctx.clone();

        tokio::spawn(async move {
            println!("Spawned thread for contest: {}", contest.name);

            let sleep_duration = contest.start_time - NOTIFICATION_BEFORE - Utc::now();
            println!(
                "{} starts at {} ({}), waiting for {} seconds",
                contest.name,
                contest.start_time,
                contest.start_time - NOTIFICATION_BEFORE,
                sleep_duration.num_seconds()
            );
            sleep_until(Instant::now() + sleep_duration.to_std().unwrap()).await;

            let data = load().unwrap();
            let channel = (*data.channel.lock().unwrap())
                .context("Channel not set")
                .unwrap();
            let mention = *data.mention.lock().unwrap();

            let embed = CreateEmbed::default()
                .title(format!("まもなく{}が始まります！", contest.name))
                .url(&contest.url)
                .field(
                    "開始時刻",
                    format!(
                        "<t:{timestamp}:F>(<t:{timestamp}:R>)",
                        timestamp = contest.start_time.timestamp()
                    ),
                    false,
                )
                .field("時間", format!("{}分", contest.duration), false)
                .field("レーティング変化", contest.rated_range, false)
                .color(Color::DARK_GREEN);

            channel
                .send_message(
                    ctx,
                    CreateMessage::default()
                        .content(mention.map_or("".to_string(), |m| m.mention().to_string()))
                        .embed(embed),
                )
                .await
                .unwrap();
        });
    }

    save(&data)?;

    Ok(())
}
