use crate::ShardManagerContainer;
use serenity::client::bridge::gateway::ShardId;
use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use chrono::prelude::*;
use std::time::UNIX_EPOCH;

#[command]
#[aliases("status", "ping", "info")]
#[description("Prints info about the bot, including shard number and ping.")]
#[bucket("normal")]
async fn about(ctx: &Context, msg: &Message) -> CommandResult {
    let now = Utc::now();
    let ping = &format!(
        "{}",
        (DateTime::<Utc>::from(UNIX_EPOCH) + now.signed_duration_since(msg.timestamp)).format("%f")
    );
    let ping = &format!("{}ms", &ping[..ping.len() - 6].parse::<u32>().unwrap());

    let data = ctx.data.read().await;

    let shard_manager = match data.get::<ShardManagerContainer>() {
        Some(v) => v,
        None => {
            msg.reply(ctx, "There was a problem getting the shard manager.")
                .await?;

            return Ok(());
        }
    };

    let manager = shard_manager.lock().await;
    let runners = manager.runners.lock().await;

    let runner = match runners.get(&ShardId(ctx.shard_id)) {
        Some(runner) => runner,
        None => {
            msg.reply(ctx, "No shard found").await?;

            return Ok(());
        }
    };

    let latency = match runner.latency {
        Some(v) => format!("{} ms", v.as_millis()),
        None => "N/A".to_string(),
    };

    let a = msg
        .channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.colour(0xB330FF)
                    .title("About")
                    .description("Massive is a simple utilities bot made by bigspeed.")
                    .field("🪞 Shard", format!("{}/2", ctx.shard_id), true)
                    .field("⏱️ Ping", ping, true)
                    .field("❤️ Heartbeat", &format!("{}", latency), true)
                    .footer(|f| f.text(&format!("Requested by {}", msg.author.name)))
                    .timestamp(format!("{}", Utc::now().format("%+")))
            })
        })
        .await;

    match a {
        Err(why) => Err(From::from(why)),
        _ => Ok(()),
    }
}
