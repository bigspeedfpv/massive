use massive::{get_server, update_server};
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::time::Duration;

#[command]
#[description("Shows the settings for the guild.")]
#[required_permissions(MANAGE_GUILD)]
#[only_in(guilds)]
#[sub_commands(set)]
#[owner_privilege]
async fn settings(ctx: &Context, msg: &Message) -> CommandResult {
    let server_settings = get_server(msg.guild_id.unwrap().into());

    let server_name = msg.guild_id.unwrap().name(ctx).await.unwrap();

    let react = if server_settings.react { "✅" } else { "❌" };

    let response = msg
        .channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.title(&format!("Settings for {}", server_name))
                    .color(0xB330FF)
                    .description(&format!(
                        "
            **Guild Prefix** (prefix): {}
            **React to Commands** (react): {}
        ",
                        server_settings.prefix, react
                    ))
            })
        })
        .await;

    match response {
        Ok(_) => Ok(()),
        Err(_) => Err(From::from("Failed to send embed for settings")),
    }
}

#[command]
#[description(
    "Changes a setting in the current guild. Setting names are specified in parentheses."
)]
#[usage("<setting> [value]")]
async fn set(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        return Err(From::from("You must provide a setting to change!"));
    }

    let mut server = get_server(msg.guild_id.unwrap().into());

    let setting = args.single::<String>().unwrap();

    let val_to_set;

    if let Ok(value) = args.single::<String>() {
        val_to_set = value
    } else {
        let help = match &setting[..] {
            "prefix" => "Prefix must be at most 5 characters.",
            "react" => "Valid options are `true` or `false`.",
            _ => {
                return Err(From::from(
                    "Unrecognized setting! Please run `settings` to view available options.",
                ))
            }
        };

        let prompt = msg
            .channel_id
            .send_message(ctx, |m| {
                m.embed(|e| {
                    e.title(&format!("Please enter the value for {}.", setting))
                        .description(help)
                        .color(0xB330FF)
                })
            })
            .await;

        if let Some(prompt_res) = msg
            .channel_id
            .await_reply(&ctx)
            .timeout(Duration::from_secs(10))
            .author_id(msg.author.id)
            .await
        {
            val_to_set = prompt_res.content.clone();
        } else {
            if let Ok(prompt_msg) = prompt {
                let _ = prompt_msg.delete(ctx).await;
            }

            return Err(From::from("No reponse within 10 seconds, cancelled."));
        }
    }

    if setting == "prefix".to_string() && val_to_set.len() > 5 {
        return Err(From::from("The prefix must be at most 5 characters."));
    }

    match &setting[..] {
        "prefix" => server.prefix = val_to_set,
        "react" => match &val_to_set[..] {
            "true" => server.react = true,
            "false" => server.react = false,
            _ => return Err(From::from("React must be either `true` or `false`.")),
        },
        _ => return Err(From::from("That is not a valid setting!")),
    }

    update_server(server);

    Ok(())
}
