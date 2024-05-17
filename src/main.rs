use twitch_irc::login::RefreshingLoginCredentials;
use twitch_irc::message::{IRCMessage, ServerMessage};
use twitch_irc::{TwitchIRCClient, ClientConfig, SecureTCPTransport};

use dotenv::dotenv;
use std::env;

mod oauth;
use oauth::OAuthStorage;

mod twitch_api_handler;


#[tokio::main]
pub async fn main() {
    const CHAT_CHANNEL: &str = "#minotwar";

    dotenv().ok();
    let client_id = env::var("TWITCH_CLIENT_ID".to_owned()).unwrap();
    let client_secret = env::var("TWITCH_CLIENT_SECRET".to_owned()).unwrap();
    let oauth_string = env::var("TWITCH_OAUTH".to_owned()).unwrap();

    let storage = OAuthStorage::new(client_id.clone(), client_secret.clone(), oauth_string).await.unwrap();
    let credentials = RefreshingLoginCredentials::init(client_id, client_secret, storage);

    let config = ClientConfig::new_simple(credentials);
    let (mut incoming_messages, client) = 
        TwitchIRCClient::<SecureTCPTransport, RefreshingLoginCredentials<OAuthStorage>>::new(config);

    client.join(CHAT_CHANNEL.to_string()).unwrap();

    let client_clone = client.clone();
    let join_handle = tokio::spawn(async move {
        while let Some(message) = incoming_messages.recv().await {
            match message {
                ServerMessage::Privmsg(msg) => {
                    let (join_requested, is_connected) = client_clone.get_channel_status(CHAT_CHANNEL.to_string()).await;
                    if msg.message_text.contains("test") && join_requested && is_connected {
                        let _ = client_clone.say(CHAT_CHANNEL.to_string(), String::from("I'm alive!")).await;
                        println!("AWWWWWW YEEEEEAAAAAHHHHHH");
                    }
                    println!("PRIVMSG -> {} {}", msg.channel_login, msg.sender.name);
                },
                other_message => {
                    let msg = IRCMessage::from(other_message);
                    println!("{:?}", msg);
                }
            }
        }
    });
    

    join_handle.await.unwrap();
}
