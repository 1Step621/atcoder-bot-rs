use std::collections::HashMap;

use crate::{
    api_parsing::{difficulty, types::*},
    load,
};
use anyhow::{Context, Error};
use chrono::{Duration, Local, NaiveTime};
use poise::serenity_prelude as serenity;
use reqwest::{
    Client,
    header::{ACCEPT_ENCODING, HeaderMap},
};
use serde::Deserialize;
use serenity::{CreateEmbed, CreateMessage};

pub async fn list_submission(ctx: &serenity::Context) -> Result<(), Error> {
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
                            let diff = difficulty::normalize(d);
                            format!("{}({})", diff, difficulty::Color::from(diff))
                        })
                        .unwrap_or("不明".into()),
                    self.language,
                    self.submission_url
                ),
                false,
            )
        }
    }

    let data = load()?;
    let users = data.users.lock().unwrap().clone();
    let channel = (*data.channel.lock().unwrap()).context("Channel not set")?;

    async fn http_get<T: for<'de> Deserialize<'de>>(url: &str) -> Result<T, Error> {
        let client = Client::new();
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT_ENCODING, "gzip".parse().unwrap());
        let res = client
            .get(url)
            .headers(headers)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        Ok(serde_json::from_str::<T>(&res)?)
    }

    let problem_models: HashMap<String, ProblemModelItem> =
        http_get("https://kenkoooo.com/atcoder/resources/problem-models.json").await?;
    let problems: Vec<ProblemItem> =
        http_get("https://kenkoooo.com/atcoder/resources/problems.json").await?;

    let mut embeds = vec![];
    for user in users {
        println!("Processing user: {}", user);

        let from = (Local::now() - Duration::days(1))
            .with_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
            .unwrap();
        let to = Local::now()
            .with_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
            .unwrap();

        let submissions_url = format!(
            "https://kenkoooo.com/atcoder/atcoder-api/v3/user/submissions?user={}&from_second={}",
            user,
            from.timestamp()
        );
        let submissions: Vec<SubmissionItem> = http_get(&submissions_url).await?;

        let accept_submissions = submissions
            .iter()
            .filter(|&s| s.epoch_second < to.timestamp())
            .filter(|s| s.result == "AC")
            .collect::<Vec<_>>();

        let accept_details = accept_submissions
            .iter()
            .map(|submission| {
                let problem_model = problem_models
                    .get(&submission.problem_id)
                    .cloned()
                    .unwrap_or_default();
                let problem = problems
                    .iter()
                    .find(|p| p.id == submission.problem_id)
                    .cloned()
                    .unwrap_or_default();
                ProblemDetail {
                    title: problem.title.clone(),
                    difficulty: problem_model.difficulty,
                    language: submission.language.clone(),
                    submission_url: format!(
                        "https://atcoder.jp/contests/{}/submissions/{}",
                        submission.contest_id, submission.id
                    ),
                }
            })
            .collect::<Vec<_>>();

        embeds.extend(accept_details.chunks(25).map(|accepts| {
            CreateEmbed::default()
                .title(format!("{} さんが昨日ACした問題", user))
                .url(format!("https://atcoder.jp/users/{}", user))
                .fields(accepts.iter().map(|p| p.to_field()))
                .color(u32::from(
                    accepts
                        .iter()
                        .map(|p| {
                            p.difficulty
                                .map(difficulty::normalize)
                                .map(difficulty::Color::from)
                                .unwrap_or(difficulty::Color::Black)
                        })
                        .max()
                        .unwrap(),
                ))
        }));
    }

    if embeds.is_empty() {
        channel
            .send_message(
                &ctx,
                CreateMessage::default().content("昨日は誰もACしませんでした。"),
            )
            .await?;
    } else {
        for embeds in embeds.chunks(10) {
            channel
                .send_message(&ctx, CreateMessage::default().embeds(embeds.to_vec()))
                .await?;
        }
    }

    Ok(())
}
