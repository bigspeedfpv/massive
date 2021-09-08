use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

#[command]
#[description("Blacklists (silently bans) a user from the current server.")]
#[usage("<mention | id>")]
#[aliases("pardon")]
#[required_permissions(BAN_MEMBERS)]
async fn blacklist(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
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

    if let Some(current_guild) = msg.guild(ctx).await {
        let ban_result = current_guild
            .ban_with_reason(ctx, id, 0, "Blacklisted by Massive")
            .await;

        if let Ok(_) = ban_result {
            Ok(())
        } else {
            Err(From::from(
                "Failed to ban user. Do I have the correct permissions?",
            ))
        }
    } else {
        Err(From::from(format!(
            "Couldn't get guild info! {:?}",
            msg.guild(ctx).await
        )))
    }
}
