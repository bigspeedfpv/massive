use serenity::framework::standard::{macros::command, CommandResult, Args};
use serenity::model::prelude::*;
use serenity::prelude::*;

#[command]
#[description("Repeats the input it is given.")]
async fn say(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
  let _ = msg.channel_id.send_message(ctx, |f| f.content(args.rest())).await;

  let _ = msg.delete(ctx).await;

  Ok(())
}