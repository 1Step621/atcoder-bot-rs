use crate::{notify, save, Context};
use anyhow::Error;
use itertools::Itertools;
use poise::serenity_prelude as serenity;
use serenity::Mentionable;

/// メッセージを送信するチャンネルを設定します。
#[poise::command(slash_command)]
pub async fn channel(ctx: Context<'_>) -> Result<(), Error> {
    {
        ctx.data().channel.lock().unwrap().replace(ctx.channel_id());
        save(ctx.data())?;
    }
    ctx.reply(format!(
        "チャンネルを {} に設定しました。",
        ctx.channel_id().mention()
    ))
    .await?;
    println!("Channel set: {:?}", ctx.channel_id());
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
    notify::notify(ctx.serenity_context().clone()).await?;
    ctx.reply("完了！").await?;
    Ok(())
}
