use chrono::Utc;
use serenity::framework::standard::{macros::command, CommandResult, Args};
use serenity::model::prelude::*;
use serenity::prelude::*;

#[command]
#[description("Bans a user from the current server.")]
#[usage("<mention | id> [reason]")]
#[aliases("pardon")]
#[required_permissions(MANAGE_MESSAGES)]
async fn ban(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  let id: UserId;

  if args.len() == 0 {
    return Err(From::from("Please mention or enter the ID of the user you would like to ban (e.g. `ban 277822562116042753`)."));
  }

  if msg.mentions.len() != 0 {
    id = msg.mentions[0].id;
    args.advance();
  } else {
    let user = args.single::<String>();

    if let Ok(user_id) = user?.parse::<u64>() {
      id = user_id.into();
    } else {
      return Err(From::from("That doesn't look like an ID or a mention! If it's an ID, are you sure you copied it correctly?"));
    }
  }

  let author_nick = msg.author.nick_in(ctx, msg.guild_id.unwrap()).await.unwrap_or(String::from(&msg.author.name));
  let guild_name = msg.guild_id.unwrap().name(ctx).await.unwrap();
  let mut reason = args.rest();

  if reason == "" { reason = "No reason given." }

  if let Some(current_guild) = msg.guild(ctx).await {
    if let Ok(user) = id.to_user(ctx).await {
      let ban_result = current_guild.ban_with_reason(ctx, id, 0, &reason[..]).await;

      if let Ok(_) = ban_result {
        let dm_result = user.direct_message(ctx, |f| f.embed(|e|
          e.author(|a| 
            a.icon_url(msg.author.face())
              .name(author_nick)
          )
            .title(&format!("Banned from {}", guild_name))
            .description(&format!("**Reason:** {}\n\nYou cannot rejoin unless you are unbanned.", reason))
            .timestamp(format!("{}", Utc::now().format("%+")))
        )).await;

        match dm_result {
          Ok(_) => {
            let _ = msg.reply(ctx, "<a:done:876387797030821899> Successfully banned user.").await;
          },
          Err(why) => {
            let _ = msg.reply(ctx, "<a:done:876387797030821899> Successfully banned user, but was unable to DM them. It's possible that they disabled direct messages.").await;
            println!("Failed to dm user: {}", why);
          },
        }
      } else {
        let _ = msg.reply(ctx, "<a:excl:877661330411229225> Failed to ban user. Do I have the correct permissions?").await;
        println!("Failed to ban user: {:?}", ban_result);
      }
    } else {
      println!("Couldn't get user info! {:?}", id.to_user(ctx).await);
    }
  } else {
    println!("Couldn't get guild info! {:?}", msg.guild(ctx).await);
  }

  Ok(())
}