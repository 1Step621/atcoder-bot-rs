use chrono::{Duration, Local, NaiveTime, Timelike, Utc};
use poise::serenity_prelude::*;
use tokio::time::{Instant, sleep_until};

use crate::functions::periodic::{check_upcomings, list_submission};

pub fn start_waiting(ctx: Context) {
    let ctx_notify = ctx.clone();
    let ctx_upcoming = ctx.clone();

    tokio::spawn(async move {
        loop {
            let now = Local::now();
            let target_time = {
                let res = Local::now()
                    .with_time(NaiveTime::from_hms_opt(4, 0, 0).unwrap())
                    .unwrap();
                if res < now {
                    res + Duration::days(1)
                } else {
                    res
                }
            };
            let sleep_duration = target_time - now;

            println!("[list_submission] Now: {}", now);
            println!("[list_submission] Next run: {}", target_time);
            println!(
                "[list_submission] Sleeping for {} seconds",
                sleep_duration.num_seconds()
            );

            sleep_until(Instant::now() + sleep_duration.to_std().unwrap()).await;
            list_submission::list_submission(&ctx_notify)
                .await
                .expect("Failed to run list_submission");
        }
    });

    tokio::spawn(async move {
        loop {
            check_upcomings::check_upcomings(&ctx_upcoming)
                .await
                .expect("Failed to run check_upcomings");

            let now = Utc::now();
            let target_time = {
                let mut res = Utc::now()
                    .with_minute(0)
                    .and_then(|t| t.with_second(0))
                    .and_then(|t| t.with_nanosecond(0))
                    .unwrap();
                while res < now {
                    res += Duration::minutes(10);
                }
                res
            };
            let sleep_duration = target_time - now;

            println!("[check_upcomings] Now: {}", now);
            println!("[check_upcomings] Next run: {}", target_time);
            println!(
                "[check_upcomings] Sleeping for {} seconds",
                sleep_duration.num_seconds()
            );

            sleep_until(Instant::now() + sleep_duration.to_std().unwrap()).await;
        }
    });
}
