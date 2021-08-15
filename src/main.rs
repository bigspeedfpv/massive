mod commands;

use commands::{
    general::{about::*},
    admin::{say::*, togglegif::*},
};

use dotenv::dotenv;
use std::{
    collections::{HashSet},
    env,
    sync::Arc,
};
use serenity::{
    async_trait,
    client::bridge::gateway::{ShardManager},
    framework::standard::{
        help_commands,
        macros::{group, help, hook},
        Args,
        CommandGroup,
        CommandResult,
        DispatchError,
        HelpOptions,
        StandardFramework,
    },
    http::Http,
    model::{
        channel::{Message},
        gateway::Ready,
        id::{
            UserId,
            GuildId,
        },
        prelude::Activity,
        user::OnlineStatus,
    },
    prelude::*,
};
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

// allow data access between shards
struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct GifOnlyToggle;

impl TypeMapKey for GifOnlyToggle {
    type Value = bool;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} shard {} is connected!", ready.user.name, ctx.shard_id + 1);

        ctx.shard.set_presence(Some(Activity::playing(&format!("with Rust | Shard {}/{}", ctx.shard_id + 1, 2))), OnlineStatus::Idle);
    }
}

#[group]
#[commands(about)]
struct General;

#[group]
#[commands(say, togglegif)]
#[owners_only]
struct Admin;

#[help]
#[individual_command_tip = "Hello! If you'd like to learn more about a specific command, just pass the name as an argument (e.g. `u.help status`)."]
#[command_not_found_text = "Sorry, I couldn't find the command {}."]
#[max_levenshtein_distance(3)]
#[lacking_permissions = "Hide"]
#[lacking_role = "Hide"]
#[wrong_channel = "Hide"]
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
    println!("Received command \"{}\" from user \"{}\"", command_name, msg.author.name);
    true
}

#[hook]
async fn after(_ctx: &Context, msg: &Message, command_name: &str, command_result: CommandResult) {
    match command_result {
        Ok(()) => println!("Processed command \"{}\" from user \"{}\" successfully", command_name, msg.author.name),
        Err(why) => println!("Command \"{}\" from user \"{}\" failed: {:?}", command_name, msg.author.name, why),
    }
}

#[hook]
async fn unknown_command(_ctx: &Context, _msg: &Message, unknown_command_name: &str) {
    println!("Could not find command named \"{}\"", unknown_command_name);
}

#[hook]
async fn normal_message(ctx: &Context, msg: &Message) {
    println!("Message is not a command: {}", msg.content);

    if !ctx.data.read().await.get::<GifOnlyToggle>().unwrap() { return; }

    if msg.guild_id == Some(GuildId::from(253766906824228866)) && msg.channel_id == 844318561052393502 {
        println!("Message sent in gif channel");

        if !msg.content.starts_with("https://tenor.com/") && !msg.content.starts_with("https://c.tenor.com") && !msg.content.ends_with(".gif") && msg.content != "" {
            println!("Message is not a gif, deleting.");
            let _ = msg.delete(ctx).await;

            let sent = msg.channel_id.send_message(ctx, |f| f.content("Please keep use of this channel to gifs only. For random conversation, check out <#694125599454658560>!")).await;

            match sent {
                Ok(m) => {
                    sleep(Duration::from_secs(5)).await;
                    let _ = m.delete(ctx).await;
                },
                Err(why) => { println!("Failed to delete sent message: {}", why); },
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
                .say(&ctx.http, &format!("Try this again in {} seconds.", info.as_secs()))
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
        },
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| c
            .with_whitespace(true)
            .on_mention(Some(bot_id))
            .prefix("mass ")
            .delimiters(vec![" "])
            .owners(owners))
        .before(before)
        .after(after)
        .unrecognised_command(unknown_command)
        .normal_message(normal_message)
        .on_dispatch_error(dispatch_error)
        .bucket("normal", |b| b.delay(5)).await
        .help(&CUSTOM_HELP)
        .group(&GENERAL_GROUP)
        .group(&ADMIN_GROUP);

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
        data.insert::<GifOnlyToggle>(true);
    }

    if let Err(why) = client.start_shards(2).await {
        println!("Client error: {:?}", why);
    }
}
