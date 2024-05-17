use chrono::{DateTime, Utc};
use reqwest::{Client, Request, RequestBuilder, Response};
use twitch_irc::login::UserAccessToken;

use super::oauth::OAuthStorage;


pub struct TwitchApiHandlers {
  client: Client,
  twitch_client_id: String,
  twitch_client_secret: String,
  token_auth: UserAccessToken,
}

impl TwitchApiHandlers {

  pub fn new(client_id: String, client_secret: String) -> Self {
    let token_auth = OAuthStorage::_read_token_auth();
    Self {
      client: Client::new(),
      twitch_client_id: client_id,
      twitch_client_secret: client_secret,
      token_auth
    }
  }

  async fn execute(&self, req: Request) -> Result<Response, reqwest::Error> {
    Ok(self.client.execute(req).await?)
  }

  fn update_token_auth(&mut self) {
    // tokens in token_auth file should theoretically be updating on their own thanks to the twitch_irc library
    // so make sure to update whatever is stored here
    let token_auth = OAuthStorage::_read_token_auth();
    self.token_auth = token_auth;
  }

  pub fn timeout_user(&mut self) {
  }
}