use std::collections::HashMap;

use crate::{difficulty, load};
use anyhow::{Context, Error};
use chrono::{Duration, Local};
use poise::serenity_prelude as serenity;
use reqwest::{
    header::{HeaderMap, ACCEPT_ENCODING},
    Client,
};
use serde::Deserialize;
use serenity::{CreateEmbed, CreateMessage};

pub async fn notify(ctx: serenity::Context) -> Result<(), Error> {
    #[allow(unused)]
    #[derive(Clone, Deserialize, Debug, Default)]
    struct ProblemModelItem {
        slope: Option<f64>,
        intercept: Option<f64>,
        variance: Option<f64>,
        difficulty: Option<isize>,
        discrimination: Option<f64>,
        irt_loglikelihood: Option<f64>,
        irt_users: Option<isize>,
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

    #[derive(Clone, Deserialize, Debug, PartialEq)]
    #[serde(rename_all = "UPPERCASE")]
    enum JudgeStatus {
        Ce,
        Mle,
        Tle,
        Re,
        Ole,
        Ie,
        Wa,
        Ac,
        Wj,
        Wr,
    }

    #[allow(unused)]
    #[derive(Clone, Deserialize, Debug)]
    struct SubmissionItem {
        id: isize,
        epoch_second: isize,
        problem_id: String,
        contest_id: String,
        user_id: String,
        language: String,
        point: f64,
        length: isize,
        result: JudgeStatus,
        execution_time: Option<isize>,
    }

    struct ProblemDetail {
        title: String,
        difficulty: Option<isize>,
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

    let data = load()?;
    let users = data.users.lock().unwrap().clone();
    let channel = (*data.channel.lock().unwrap()).context("Channel not set")?;

    let problem_models: HashMap<String, ProblemModelItem> =
        http_get("https://kenkoooo.com/atcoder/resources/problem-models.json").await?;
    let problems: Vec<ProblemItem> =
        http_get("https://kenkoooo.com/atcoder/resources/problems.json").await?;

    let mut embeds = vec![];
    for user in users {
        println!("Processing user: {}", user);

        let submissions_url = format!(
            "https://kenkoooo.com/atcoder/atcoder-api/v3/user/submissions?user={}&from_second={}",
            user,
            (Local::now() - Duration::days(1)).timestamp()
        );
        let submissions: Vec<SubmissionItem> = http_get(&submissions_url).await?;

        let accept_submissions = submissions
            .iter()
            .filter(|s| s.result == JudgeStatus::Ac)
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
                        problem.contest_id, submission.id
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
                ctx,
                CreateMessage::default().content("昨日は誰もACしませんでした。"),
            )
            .await?;
    } else {
        channel
            .send_message(ctx, CreateMessage::default().embeds(embeds))
            .await?;
    }

    Ok(())
}
