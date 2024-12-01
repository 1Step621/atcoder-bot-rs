use crate::notify;
use chrono::{Duration, Local, NaiveTime};
use poise::serenity_prelude as serenity;
use tokio::time::{sleep_until, Instant};

pub async fn wait(ctx: serenity::Context) {
    loop {
        let now = Local::now();
        let target_time = (Local::now() + Duration::days(1))
            .with_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
            .unwrap();
        let sleep_duration = target_time - now;

        println!("Now: {}", now);
        println!("Next run: {}", target_time);
        println!("Sleeping for {} seconds", sleep_duration.num_seconds());

        sleep_until(Instant::now() + sleep_duration.to_std().unwrap()).await;
        notify::notify(ctx.clone())
            .await
            .expect("Failed to run daily job");
    }
}
