use std::str::FromStr;

use dotenv::dotenv;
use regex::Regex;
use serenity::{
    async_trait,
    futures::StreamExt,
    model::prelude::{Presence, Ready, RoleId},
    prelude::{Context, EventHandler, GatewayIntents},
    Client,
};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, context: Context, bot: Ready) {
        let maybe_game = std::env::var("GAME_TO_MATCH");
        if maybe_game.is_err() {
            println!("You did not specify what game to look for! Failling silently");

            return;
        }

        let game = maybe_game.unwrap().trim().to_lowercase();
        let pattern = Regex::new(&game).unwrap();
        let raw_guild_id = std::env::var("GUILD_ID").unwrap();
        let guild = &bot
            .guilds
            .iter()
            .find(|guild| guild.id.to_string() == raw_guild_id)
            .unwrap();
        let role = get_role().unwrap();

        let mut members = guild.id.members_iter(&context).boxed();

        while let Some(member) = members.next().await {
            if let Err(err) = member {
                println!("Unexpected error: {err}");

                continue;
            }
            let mut member = member.unwrap();
            println!(
                "Handling initial presence for user {} ({})!",
                &member.user.name, &member.user.id
            );

            let presence = bot.presences.get(&member.user.id);
            if presence.is_none() {
                member.remove_role(&context, &role).await.unwrap();

                continue;
            }

            for activity in &presence.unwrap().activities {
                let activity_name = activity.name.trim().to_lowercase();
                println!(
                    "found activity {} (parsed into {})",
                    &activity.name, &activity_name
                );
                if pattern.is_match(&activity_name) {
                    println!("Matched!");
                    member
                        .add_role(&context, get_role().unwrap())
                        .await
                        .unwrap();

                    break;
                }
                println!("Did not match!");

                member
                    .remove_role(&context, get_role().unwrap())
                    .await
                    .unwrap();
            }
        }
    }

    async fn presence_update(&self, context: Context, presence: Presence) {
        if presence.user.bot.is_some() && presence.user.bot.unwrap() == true {
            return;
        }
        let mut member = presence
            .guild_id
            .unwrap()
            .member(&context, presence.user.id)
            .await
            .unwrap();

        println!(
            "Handling presence update for user {} ({})!",
            &presence
                .user
                .name
                .clone()
                .unwrap_or("(no username found?)".to_owned()),
            &presence.user.id
        );

        let maybe_game = std::env::var("GAME_TO_MATCH");
        if let Err(_) = maybe_game {
            println!("You did not specify what game to look for! Failling silently");

            return;
        }

        let game = maybe_game.unwrap().trim().to_lowercase();
        let pattern = Regex::new(&game).unwrap();

        if presence.activities.len() == 0 {
            member
                .remove_role(&context, get_role().unwrap())
                .await
                .unwrap();
        }

        for activity in presence.activities {
            let activity_name = activity.name.trim().to_lowercase();
            println!(
                "found activity {} (parsed into {})",
                &activity.name, &activity_name
            );
            if pattern.is_match(&activity_name) {
                println!("Matched!");
                member
                    .add_role(&context, get_role().unwrap())
                    .await
                    .unwrap();

                break;
            }
        }
    }
}

fn get_role<'a>() -> Result<RoleId, &'a str> {
    RoleId::from_str(&std::env::var("ROLE_ID").map_err(|_| "Please provide a role id")?)
        .map_err(|_| "Invalid role id")
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let bot_token = std::env::var("BOT_TOKEN").expect("Please provide a bot token!");
    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::GUILD_PRESENCES
        | GatewayIntents::GUILD_MEMBERS;

    let mut client = Client::builder(bot_token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    println!("Hello, world!");
    if let Err(reason) = client.start().await {
        println!("An error occured while running the client {:?}", reason);
    }
}
