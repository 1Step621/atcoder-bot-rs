use crate::{PoiseContext, WellKnownContest, functions::periodic::list_submission, save};
use anyhow::Error;
use itertools::Itertools;
use poise::serenity_prelude::*;

/// 提出メッセージを送信するチャンネルを設定します。
#[poise::command(slash_command)]
pub async fn set_submissions_channel(
    ctx: PoiseContext<'_>,
    #[description = "提出メッセージを送信するチャンネル"] channel: Option<Channel>,
) -> Result<(), Error> {
    let channel_id = channel.map(|c| c.id()).unwrap_or(ctx.channel_id());
    {
        ctx.data()
            .submissions_channel
            .lock()
            .unwrap()
            .replace(channel_id);
        save(ctx.data())?;
    }
    ctx.reply(format!(
        "提出メッセージチャンネルを {} に設定しました。",
        channel_id.mention()
    ))
    .await?;
    println!("Submissions channel set: {:?}", channel_id);
    Ok(())
}

/// コンテスト通知メッセージを送信するチャンネルを設定します。
#[poise::command(slash_command)]
pub async fn set_contests_channel(
    ctx: PoiseContext<'_>,
    #[description = "コンテスト通知メッセージを送信するチャンネル"] channel: Option<Channel>,
) -> Result<(), Error> {
    let channel_id = channel.map(|c| c.id()).unwrap_or(ctx.channel_id());
    {
        ctx.data()
            .contests_channel
            .lock()
            .unwrap()
            .replace(channel_id);
        save(ctx.data())?;
    }
    ctx.reply(format!(
        "コンテスト通知チャンネルを {} に設定しました。",
        channel_id.mention()
    ))
    .await?;
    println!("Contests channel set: {:?}", channel_id);
    Ok(())
}

/// AtCoderのユーザーを登録します。カンマ区切りで複数人指定できます。
#[poise::command(slash_command)]
pub async fn register(
    ctx: PoiseContext<'_>,
    #[description = "AtCoderのユーザー名"] users: String,
) -> Result<(), Error> {
    let users = users
        .split(",")
        .map(|u| u.trim().to_string())
        .collect::<Vec<_>>();
    {
        ctx.data().users.lock().unwrap().extend(users.clone());
        save(ctx.data())?;
    }
    ctx.reply(format!("ユーザー ({}) を登録しました。", users.join(", ")))
        .await?;
    println!("User registered: {:?}", &users);
    Ok(())
}

/// AtCoderのユーザーを登録解除します。
#[poise::command(slash_command)]
pub async fn unregister(
    ctx: PoiseContext<'_>,
    #[description = "AtCoderのユーザー名"] user: String,
) -> Result<(), Error> {
    {
        ctx.data().users.lock().unwrap().remove(&user);
        save(ctx.data())?;
    }
    ctx.reply(format!("ユーザー ({}) を登録解除しました。", user))
        .await?;
    println!("User unregistered: {:?}", &user);
    Ok(())
}

/// 登録されているユーザーの一覧を表示します。
#[poise::command(slash_command)]
pub async fn register_list(ctx: PoiseContext<'_>) -> Result<(), Error> {
    let users = ctx.data().users.lock().unwrap().clone();
    ctx.reply(format!(
        "登録されているユーザー: {}",
        users.iter().join(", ")
    ))
    .await?;
    Ok(())
}

/// 手動で実行します。
#[poise::command(slash_command)]
pub async fn run(ctx: PoiseContext<'_>) -> Result<(), Error> {
    ctx.defer().await?;
    list_submission::list_submission(ctx.serenity_context()).await?;
    ctx.reply("完了！").await?;
    Ok(())
}

/// コンテスト通知を設定します。
#[poise::command(slash_command)]
pub async fn enable_contest_notification(
    ctx: PoiseContext<'_>,
    #[description = "コンテストの種類"] kind: WellKnownContest,
) -> Result<(), Error> {
    {
        ctx.data().contest_kind.lock().unwrap().insert(kind);
        save(ctx.data())?;
    }
    ctx.reply(format!(
        "{}のコンテスト通知を設定しました。反映には最大1日かかる場合があります。",
        kind
    ))
    .await?;
    println!("Contest notification set: {:?}", kind);
    Ok(())
}

/// コンテスト通知を解除します。
#[poise::command(slash_command)]
pub async fn disable_contest_notification(
    ctx: PoiseContext<'_>,
    #[description = "コンテストの種類"] kind: WellKnownContest,
) -> Result<(), Error> {
    {
        ctx.data().contest_kind.lock().unwrap().remove(&kind);
        save(ctx.data())?;
    }
    ctx.reply(format!(
        "{}のコンテスト通知を解除しました。反映には最大1日かかる場合があります。",
        kind
    ))
    .await?;
    println!("Contest notification disabled: {:?}", kind);
    Ok(())
}

/// コンテスト通知の際にメンションするロールを設定します。
#[poise::command(slash_command)]
pub async fn set_mention(
    ctx: PoiseContext<'_>,
    #[description = "メンションするロール"] role: Option<Role>,
) -> Result<(), Error> {
    let role_id = role.map(|r| r.id);
    {
        *ctx.data().mention.lock().unwrap() = role_id;
        save(ctx.data())?;
    }
    ctx.reply(format!(
        "メンションするロールを {} に設定しました。",
        role_id.map_or("なし".to_string(), |r| r.mention().to_string())
    ))
    .await?;
    println!("Mention role set: {:?}", role_id);
    Ok(())
}
