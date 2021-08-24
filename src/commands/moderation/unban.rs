use chrono::Utc;
use serenity::framework::standard::{macros::command, CommandResult, Args};
use serenity::model::prelude::*;
use serenity::prelude::*;

#[command]
#[description("Unbans a user from the current server.")]
#[usage("<mention | id> [reason]")]
#[required_permissions(MANAGE_MESSAGES)]
async fn unban(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  let id: UserId;

  if args.len() == 0 {
    return Err(From::from("Please mention or enter the ID of the user you would like to unban (e.g. `unban 277822562116042753`)."));
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

  let invite_link = msg.channel_id.create_invite(ctx, |i| i.max_uses(1).max_age(0)).await;

  let invite_link = match invite_link {
    Ok(v) => format!("https://discord.gg/{}", v.code),
    Err(_) => "No invite link.".to_string(),
  };

  // prefer sending vanity url
  let vanity_url = msg.guild_id.unwrap().vanity_url(ctx).await.unwrap_or(invite_link);

  if let Some(current_guild) = msg.guild(ctx).await {
    if let Ok(user) = id.to_user(ctx).await {
      let unban_result = current_guild.unban(ctx, id).await;

      if let Ok(_) = unban_result {
        let dm_result = user.direct_message(ctx, |f| f.embed(|e|
          e.author(|a| 
            a.icon_url(msg.author.face())
              .name(author_nick)
          )
            .title(&format!("Unbanned from {}", guild_name))
            .description(&format!("**Reason:** {}\n\nYou can rejoin with the following link: {}", reason, vanity_url))
            .timestamp(format!("{}", Utc::now().format("%+")))
        )).await;

        match dm_result {
          Ok(_) => {
            let _ = msg.reply(ctx, "<a:done:876387797030821899> Successfully unbanned user.").await;
          },
          Err(why) => {
            let _ = msg.reply(ctx, "<a:done:876387797030821899> Successfully unbanned user, but was unable to DM them. It's possible that they disabled direct messages.").await;
            println!("Failed to dm user: {}", why);
          },
        }
      } else {
        let _ = msg.reply(ctx, "<a:excl:877661330411229225> Failed to unban user. Do I have the correct permissions?").await;
        println!("Failed to ban user: {:?}", unban_result);
      }
    } else {
      println!("Couldn't get user info! {:?}", id.to_user(ctx).await);
    }
  } else {
    println!("Couldn't get guild info! {:?}", msg.guild(ctx).await);
  }

  Ok(())
}