use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::{
    cache::FromStrAndCache,
    framework::standard::{
        macros::{check, command},
        Args, CommandOptions, CommandResult, Reason,
    },
};

use tokio::time::{sleep, Duration};

use crate::GifOnlyToggle;

#[command]
#[description("Toggles GIF-only mode in DRL's GIF-only channel.")]
#[required_permissions("MANAGE_MESSAGES")]
#[checks(GIFCHANNEL)]
#[bucket("normal")]
async fn togglegif(ctx: &Context, msg: &Message) -> CommandResult {
    let mut data = ctx.data.write().await;
    let gif_mode = data
        .get_mut::<GifOnlyToggle>()
        .expect("Expected GifOnlyToggle in TypeMap.");
    *gif_mode ^= true;

    let reaction = match gif_mode {
        true => "<:enabled:876374627746717736>",
        false => "<:disabled:876374627687993374>",
    };

    let _ = msg
        .react(
            ctx,
            ReactionType::from(EmojiIdentifier::from_str(ctx, reaction).await.unwrap()),
        )
        .await;

    sleep(Duration::from_secs(5)).await;

    let _ = msg.delete(ctx).await;

    Ok(())
}

#[check]
#[name("GIFChannel")]
#[display_in_help]
#[check_in_help]
async fn gif_channel_check(
    _: &Context,
    msg: &Message,
    _: &mut Args,
    _: &CommandOptions,
) -> Result<(), Reason> {
    if msg.guild_id != Some(GuildId::from(253766906824228866))
        || msg.channel_id != 844318561052393502
    {
        return Err(Reason::Log(
            "This command must be run in the GIFs only channel!".to_string(),
        ));
    }

    Ok(())
}
