mod commands;
use commands::{
    admin::{say::*, togglegif::*},
    general::{about::*, user::*},
    moderation::{ban::*, blacklist::*, kick::*, purge::*, settings::*, unban::*},
};

mod util;
use util::log;

use dotenv::dotenv;
use serenity::{
    async_trait,
    cache::FromStrAndCache,
    client::bridge::gateway::ShardManager,
    framework::standard::{
        help_commands,
        macros::{group, help, hook},
        Args, CommandGroup, CommandResult, DispatchError, HelpOptions, StandardFramework,
    },
    http::Http,
    model::{
        channel::{Message, ReactionType},
        gateway::Ready,
        id::{GuildId, UserId},
        misc::EmojiIdentifier,
        prelude::Activity,
        user::OnlineStatus,
    },
    prelude::*,
};
use std::{collections::HashSet, env, sync::Arc};

use reqwest;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

use chrono::Utc;

// allow data access between shards
struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct GifOnlyToggle;

impl TypeMapKey for GifOnlyToggle {
    type Value = bool;
}

struct ReqwestContainer;

impl TypeMapKey for ReqwestContainer {
    type Value = reqwest::Client;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        log::connected(&format!(
            "{} shard {} is connected!",
            ready.user.name,
            ready.shard.unwrap()[0] + 1
        ));

        ctx.shard.set_presence(
            Some(Activity::playing(&format!(
                "with Rust | Shard {}/{}",
                ready.shard.unwrap()[0] + 1,
                ready.shard.unwrap()[1]
            ))),
            OnlineStatus::Idle,
        );
    }
}

#[group]
#[commands(about, user)]
#[summary("General-purpose commands")]
struct General;

#[group]
#[commands(ban, blacklist, kick, settings, unban, purge, whitelist)]
#[owner_privilege(false)]
#[summary("Moderation utilities")]
struct Moderation;

#[group]
#[commands(say, togglegif)]
#[owners_only]
#[summary("Commands related to the core functionality of MASSIVE")]
struct Admin;

#[help]
#[individual_command_tip("Hello! If you'd like to learn more about a specific command, just pass the name as an argument (e.g. `help status`).")]
#[command_not_found_text("Sorry, I couldn't find the command {}.")]
#[max_levenshtein_distance(3)]
#[lacking_permissions("strike")]
#[lacking_role("strike")]
#[wrong_channel("strike")]
#[lacking_ownership("hide")]
#[embed_success_colour("#42f56f")]
async fn custom_help(
    ctx: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(ctx, msg, args, help_options, groups, owners).await;

    Ok(())
}

#[hook]
async fn before(_ctx: &Context, msg: &Message, command_name: &str) -> bool {
    log::received(&format!(
        "Received command \"{}\" from user \"{}\"",
        command_name, msg.author.name
    ));

    true
}

#[hook]
async fn after(ctx: &Context, msg: &Message, command_name: &str, command_result: CommandResult) {
    match command_result {
        Ok(()) => {
            log::success(&format!(
                "Processed command \"{}\" from user \"{}\" successfully",
                command_name, msg.author.name
            ));

            let server = massive::get_server(msg.guild_id.unwrap_or(0.into()).into());

            if server.react {
                let _ = msg
                    .react(
                        ctx,
                        ReactionType::from(
                            EmojiIdentifier::from_str(ctx, "<a:done:876387797030821899>")
                                .await
                                .unwrap(),
                        ),
                    )
                    .await;
            }
        }
        Err(why) => {
            log::error(&format!(
                "Command \"{}\" from user \"{}\" failed: {:?}",
                command_name, msg.author.name, why
            ));

            let server = massive::get_server(msg.guild_id.unwrap_or(0.into()).into());

            if server.react {
                let _ = msg
                    .react(
                        ctx,
                        ReactionType::from(
                            EmojiIdentifier::from_str(ctx, "<a:excl:877661330411229225>")
                                .await
                                .unwrap(),
                        ),
                    )
                    .await;
            }

            let _ = msg.channel_id.send_message(ctx, |m| m
                .embed(|e| e
                    .title(&format!("Command `{}` failed.", command_name))
                    .colour(0xFF3045)
                    .description(&format!("{}\n\n**Please refer to the help for this command for more info** (`help {}`).", why, command_name))
                    .footer(|f| f
                        .text("If you believe this is an error, please contact bigspeed.")
                    )
                    .timestamp(format!("{}", Utc::now().format("%+")))
                )
            ).await;
        }
    }
}

#[hook]
async fn unknown_command(ctx: &Context, msg: &Message, unknown_command_name: &str) {
    let _ = msg
        .channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.title(&format!(
                    "Unable to find command `{}`.",
                    unknown_command_name
                ))
                .colour(0xFF3045)
                .description(&format!(
                    "**Please refer to the help for more info** (`{}help`).",
                    massive::get_server(msg.guild_id.unwrap_or(0.into()).into()).prefix
                ))
                .footer(|f| f.text("If you believe this is an error, please contact bigspeed."))
                .timestamp(format!("{}", Utc::now().format("%+")))
            })
        })
        .await;

    log::error(&format!(
        "Could not find command named \"{}\"",
        unknown_command_name
    ));
}

#[hook]
async fn normal_message(ctx: &Context, msg: &Message) {
    log::info(&format!("Message is not a command: {}", msg.content));

    if !ctx.data.read().await.get::<GifOnlyToggle>().unwrap() {
        return;
    }

    if msg.guild_id == Some(GuildId::from(253766906824228866))
        && msg.channel_id == 844318561052393502
    {
        log::info(&format!("Message sent in gif channel"));

        if !msg.content.starts_with("https://tenor.com/")
            && !msg.content.starts_with("https://c.tenor.com")
            && !msg.content.ends_with(".gif")
            && msg.content != ""
        {
            log::info(&format!("Message is not a gif, deleting."));
            let _ = msg.delete(ctx).await;

            let sent = msg.channel_id.send_message(ctx, |f| f.content("Please keep use of this channel to gifs only. For random conversation, check out <#694125599454658560>!")).await;

            match sent {
                Ok(m) => {
                    sleep(Duration::from_secs(5)).await;
                    let _ = m.delete(ctx).await;
                }
                Err(why) => {
                    log::error(&format!("Failed to delete sent message: {}", why));
                }
            }
        }
    }
}

#[hook]
async fn delay_action(ctx: &Context, msg: &Message) {
    let _ = msg.react(ctx, '⏱').await;
}

#[hook]
async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError) {
    if let DispatchError::Ratelimited(info) = error {
        if info.is_first_try {
            let _ = msg
                .channel_id
                .say(
                    &ctx.http,
                    &format!("Try this again in {} seconds.", info.as_secs()),
                )
                .await;
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = env::var("GIFBOT_RS_TOKEN").expect("Expected a token in the environment");

    let http = Http::new_with_token(&token);

    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(team) = info.team {
                owners.insert(team.owner_user_id);
            } else {
                owners.insert(info.owner.id);
            }
            match http.get_current_user().await {
                Ok(bot_id) => (owners, bot_id.id),
                Err(why) => panic!("Could not access the bot id: {:?}", why),
            }
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| {
            c.with_whitespace(true)
                .on_mention(Some(bot_id))
                .prefix(massive::DEFAULT_PREFIX)
                .dynamic_prefix(|_, msg| {
                    Box::pin(async move {
                        Some(massive::get_server(msg.guild_id.unwrap_or(0.into()).into()).prefix)
                    })
                })
                .no_dm_prefix(true)
                .owners(owners)
        })
        .before(before)
        .after(after)
        .unrecognised_command(unknown_command)
        .normal_message(normal_message)
        .on_dispatch_error(dispatch_error)
        .bucket("normal", |b| b.delay(5))
        .await
        .help(&CUSTOM_HELP)
        .group(&GENERAL_GROUP)
        .group(&MODERATION_GROUP)
        .group(&ADMIN_GROUP);

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Err creating client");

    let rq_client = reqwest::Client::new();

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
        data.insert::<GifOnlyToggle>(true);
        data.insert::<ReqwestContainer>(rq_client);
    }

    if let Err(why) = client.start_shards(2).await {
        log::error(&format!("Client error: {:?}", why));
    }
}
