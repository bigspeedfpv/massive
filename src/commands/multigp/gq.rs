use serenity::framework::standard::{Args, CommandOptions, CommandResult, Reason, macros::{check, command}};
use serenity::model::prelude::*;
use serenity::prelude::*;

use serde::Deserialize;

use crate::ReqwestContainer;

#[command]
#[aliases("globalqualifier", "qualifier")]
#[description("Shows info from the GQ leaderboard. Top shows the top pilots on the leaderboard, pilot shows the position of a pilot, and place shows the pilots in a given position.")]
#[usage("gq <top | pilot | place> <query>")]
#[checks("ARGS")]
async fn gq(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
  let data = ctx.data.read().await;

  let rq_client = match data.get::<ReqwestContainer>() {
    Some(v) => v,
    None => { 
      println!("Error: Could not get ReqwestContainer");
      return Ok(());
    }
  };

  let raw_leaders_req = rq_client.get("https://web.scraper.workers.dev/?url=https%3A%2F%2Fwww.multigp.com%2Fmgp%2Fprotected%2Fmodules%2Fmultigp%2Fviews%2Frace%2F2021regionalQualifierStandings.php&selector=td&scrape=text");

  let raw_leaders = raw_leaders_req.send();
  let raw_leaders = raw_leaders.await?.text().await?;

  let leaderboard_de: RawLeaderboard = serde_json::from_str(&raw_leaders).unwrap();

  let mut leaderboard_de = leaderboard_de.result.td.into_iter().peekable();

  let mut leaderboard_formatted = Leaderboard { pilots: Vec::new() };

  while leaderboard_de.peek() != None {
    let handle_pos = leaderboard_de.next().unwrap();

    let pilot_handle = handle_pos.split(" ").collect::<Vec<&str>>()[1..].join(" ");
    let pilot_position = handle_pos.split(" ").collect::<Vec<&str>>()[0].parse::<i32>().unwrap();

    let pilot_time = leaderboard_de.next().unwrap();
    let pilot_chapter = leaderboard_de.next().unwrap();

    leaderboard_formatted.pilots.push(Pilot { handle: pilot_handle, position: pilot_position, time: pilot_time, chapter: pilot_chapter })
  }

  println!("{:?}", leaderboard_formatted);

  Ok(())
}

#[check]
#[name("Args")]
async fn args_check(ctx: &Context, msg: &Message, args: &mut Args, _: &CommandOptions) -> Result<(), Reason> {
  let mode = args.single::<String>();

  let no_mode = || async {
    let _ = msg.reply(ctx, "Please specify a mode (top, pilot, or place). Please run `help gq` for more info.").await;
    Err(Reason::Log("Please specify a mode (top, pilot, or place). Please run `help gq` for more info.".to_string()))
  };

  match mode {
    Err(_) => {
      return no_mode().await;
    },
    Ok(v) => {
      if v != "top" && v != "pilot" && v != "place" {
        return no_mode().await;
      }

      let query = args.single::<String>();

      match query {
        Err(_) => {
          let _ = msg.reply(ctx, "Please enter a query!").await;
          return Err(Reason::Log("user didn't enter query".to_string()));
        },
        Ok(v) => {
          match v.parse::<i32>() {
            Ok(v) => (),
            Err(_) => {

            }
          }
        }
      }
      
      if v == "top" {
        match query.parse::<i32>() {

        }
      }
    },
  }

  Ok(())
}

#[derive(Deserialize, Debug)]
struct RawLeaderboard {
  result: TableData,
}

#[derive(Deserialize, Debug)]
struct TableData {
  td: Vec<String>,
}

#[derive(Debug)]
struct Pilot {
  handle: String,
  position: i32,
  time: String,
  chapter: String,
}

#[derive(Debug)]
struct Leaderboard {
  pilots: Vec<Pilot>,
}