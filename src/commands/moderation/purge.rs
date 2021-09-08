use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

#[command]
#[description("Purges a set number of messages in a channel or from a user. Defaults to purging the current channel unless `user` is specified.")]
#[usage("<message count> [id | mention]")]
async fn purge(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if let Ok(message_count) = args.single::<i32>() {
        Ok(())
    } else {
        Err(From::from(
            "Unable to parse message count. Are you sure it is a number?",
        ))
    }
}
