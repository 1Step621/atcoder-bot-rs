use crate::{Context, WellKnownContest, functions::periodic::list_submission, save};
use anyhow::Error;
use itertools::Itertools;
use poise::serenity_prelude as serenity;
use serenity::Mentionable;

/// メッセージを送信するチャンネルを設定します。
#[poise::command(slash_command)]
pub async fn channel(
    ctx: Context<'_>,
    #[description = "メッセージを送信するチャンネル"] channel: Option<serenity::Channel>,
) -> Result<(), Error> {
    let channel_id = channel.map(|c| c.id()).unwrap_or(ctx.channel_id());
    {
        ctx.data().channel.lock().unwrap().replace(channel_id);
        save(ctx.data())?;
    }
    ctx.reply(format!(
        "チャンネルを {} に設定しました。",
        channel_id.mention()
    ))
    .await?;
    println!("Channel set: {:?}", channel_id);
    Ok(())
}

/// AtCoderのユーザーを登録します。カンマ区切りで複数人指定できます。
#[poise::command(slash_command)]
pub async fn register(
    ctx: Context<'_>,
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
    ctx: Context<'_>,
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
pub async fn registerlist(ctx: Context<'_>) -> Result<(), Error> {
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
pub async fn run(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;
    list_submission::list_submission(ctx.serenity_context()).await?;
    ctx.reply("完了！").await?;
    Ok(())
}

/// コンテスト通知を設定します。
#[poise::command(slash_command)]
pub async fn enable_contest_notification(
    ctx: Context<'_>,
    #[description = "コンテストの種類"] kind: WellKnownContest,
) -> Result<(), Error> {
    {
        ctx.data().contest_kind.lock().unwrap().insert(kind);
        save(ctx.data())?;
    }
    ctx.reply(format!("{}のコンテスト通知を設定しました。", kind))
        .await?;

    println!("Contest notification set: {:?}", kind);
    Ok(())
}

/// コンテスト通知を解除します。
#[poise::command(slash_command)]
pub async fn disable_contest_notification(
    ctx: Context<'_>,
    #[description = "コンテストの種類"] kind: WellKnownContest,
) -> Result<(), Error> {
    {
        ctx.data().contest_kind.lock().unwrap().remove(&kind);
        save(ctx.data())?;
    }
    ctx.reply(format!("{}のコンテスト通知を解除しました。", kind))
        .await?;

    println!("Contest notification disabled: {:?}", kind);
    Ok(())
}

/// コンテスト通知の際にメンションするロールを設定します。
#[poise::command(slash_command)]
pub async fn set_mention(
    ctx: Context<'_>,
    #[description = "メンションするロール"] role: Option<serenity::Role>,
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
