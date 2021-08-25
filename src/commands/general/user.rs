use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use chrono::Utc;

#[command]
#[description("Shows info about a user, such as their profile picture and the date their account was created.")]
#[usage("<id | mention>")]
async fn user(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let id: UserId;

    if args.len() == 0 {
        return Err(From::from("Please mention or enter the ID of the user you would like to see (e.g. `user 277822562116042753`)."));
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

    if let Ok(user) = id.to_user(ctx).await {
        let mut name = user.name.clone();
        let mut face = user.face();
        let mention = format!("<@{}>", id);
        let mut joined = String::new();
        let mut roles = String::new();
        let mut pending = "";
        let mut nitro_since = String::new();

        let created = format!(
            "<t:{timestamp}:D>, <t:{timestamp}:R>",
            timestamp = id.created_at().timestamp()
        );

        // try to get member info for user
        if let Some(guild) = msg.guild_id {
            if let Ok(member) = guild.member(ctx, id).await {
                name = member.display_name().to_string();
                face = member.face();
                pending = match member.pending {
                    true => "\nPending: True",
                    false => "",
                };

                if let Some(nitro_date) = member.premium_since {
                    nitro_since = format!(
                        "\nNitro Sub Since: <t:{timestamp}:D>, <t:{timestamp}:R>",
                        timestamp = nitro_date.timestamp()
                    )
                }

                if let Some(joined_at) = member.joined_at {
                    joined = format!(
                        "\nJoined Guild: <t:{timestamp}:D>, <t:{timestamp}:R>",
                        timestamp = joined_at.timestamp()
                    )
                }

                for role in member.roles {
                    roles.push_str(&format!("<@&{}> ", role));
                }
            }
        }

        let _ = msg
            .channel_id
            .send_message(ctx, |m| {
                m.embed(|e| {
                    e.title(name).thumbnail(face).field(
                        "About",
                        &format!(
                            "Mention: {}\nJoined: {}{}{}{}",
                            mention, created, nitro_since, joined, pending
                        ),
                        false,
                    );

                    if roles != String::from("") {
                        e.field("Roles", roles, false);
                    }

                    e.footer(|f| f.text(&format!("Requested by {}", msg.author.name)))
                        .timestamp(format!("{}", Utc::now().format("%+")));

                    e
                })
            })
            .await;

        Ok(())
    } else {
        Err(From::from("Unable to find the provided user."))
    }
}
