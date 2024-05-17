use async_trait::async_trait;
use reqwest::Client;
use twitch_irc::login::{TokenStorage, UserAccessToken, GetAccessTokenResponse};
use chrono::DateTime;

use std::collections::HashMap;
use std::fs;


#[derive(Debug, Clone)]
pub struct OAuthStorage { pub user_access_token: UserAccessToken }

impl OAuthStorage {
  const TOKEN_AUTH_FILE: &'static str = "token_auth.txt";

  pub async fn new(client_id: String, client_secret: String, oauth_string: String) -> Result<Self, reqwest::Error> {
    let user_access_token = OAuthStorage::_request_access_token(
      client_id.clone(),
      client_secret.clone(),
      oauth_string.clone(),
      false
    ).await?;
    OAuthStorage::_save_token_auth(&user_access_token);
    
    Ok(OAuthStorage { user_access_token, })
  }

  async fn _request_access_token(client_id: String, client_secret: String, token_string: String, is_refresh_token: bool) -> Result<UserAccessToken, reqwest::Error> {
    let req_url = "https://id.twitch.tv/oauth2/token";
    let mut req_body: HashMap<&str, String> = HashMap::from([
      ("client_id", client_id),
      ("client_secret", client_secret),
    ]);
  
    if is_refresh_token {
      req_body.insert("grant_type", "refresh_token".to_owned());
      req_body.insert("refresh_token", token_string);
    }
    else {
      req_body.insert("code", token_string);
      req_body.insert("grant_type", "authorization_code".to_owned());
      req_body.insert("redirect_uri", "localhost:3000".to_owned());
    }
  
    let response = Client::new()
      .post(req_url)
      .form(&req_body)
      .send().await?
      .text().await?;
    let decoded: GetAccessTokenResponse = serde_json::from_str(response.as_str()).unwrap();
    let user_access_token: UserAccessToken = UserAccessToken::from(decoded);
  
    Ok(user_access_token)
  }

  fn _save_token_auth(user_access_token: &UserAccessToken) {
    let UserAccessToken {
      access_token,
      refresh_token,
      expires_at,
      created_at
    } = user_access_token.to_owned();

    let expires_at = expires_at.unwrap().to_rfc3339(); // Ok to overwrite since we don't need a date object
    let created_at = created_at.to_rfc3339();
    let new_contents = format!(
      "TWITCH_ACCESS_TOKEN={access_token}
      TWITCH_REFRESH_TOKEN={refresh_token}
      TWITCH_TOKEN_EXPIRES_AT={expires_at}
      TWITCH_TOKEN_CREATED_AT={created_at}"
    );

    fs::write(OAuthStorage::TOKEN_AUTH_FILE, new_contents)
      .expect(format!("Should have saved new token auth values to '{}' located in root", OAuthStorage::TOKEN_AUTH_FILE).as_str())
  }

  pub fn _read_token_auth() -> UserAccessToken {
    let contents = fs::read_to_string(OAuthStorage::TOKEN_AUTH_FILE)
      .expect(format!("Should have loaded token auth values from '{}' located in root", OAuthStorage::TOKEN_AUTH_FILE).as_str());
    let mut key_value_iterable = contents.split_ascii_whitespace();

    let access_token = key_value_iterable.next()
      .unwrap()
      .split_once("=")
      .unwrap_or(("TWITCH_ACCESS_TOKEN", "")).1;

    let refresh_token = key_value_iterable.next()
      .unwrap()
      .split_once("=")
      .unwrap_or(("TWITCH_REFRESH_TOKEN", "")).1;

    let expires_at = key_value_iterable.next()
      .unwrap()
      .split_once("=")
      .unwrap_or(("TWITCH_TOKEN_EXPIRES_AT", "")).1;
    let expires_at = DateTime::parse_from_rfc3339(expires_at)
      .unwrap_or_default()
      .to_utc();

    let created_at = key_value_iterable.next()
      .unwrap()
      .split_once("=")
      .unwrap_or(("TWITCH_TOKEN_CREATED_AT", "")).1;
    let created_at = DateTime::parse_from_rfc3339(created_at)
      .unwrap_or_default()
      .to_utc();

    UserAccessToken {
      access_token: access_token.to_owned(),
      refresh_token: refresh_token.to_owned(),
      expires_at: Some(expires_at),
      created_at,
    }
  }
}


#[async_trait]
impl TokenStorage for OAuthStorage {
  type LoadError = std::io::Error;
  type UpdateError = std::io::Error;

  async fn load_token(&mut self) -> Result<UserAccessToken, Self::LoadError> {
    let current_token = OAuthStorage::_read_token_auth();
    Ok(current_token)
  }

  async fn update_token(&mut self, token: &UserAccessToken) -> Result<(), Self::UpdateError> {
    let new_token = token.to_owned();
    
    if self.user_access_token.access_token != new_token.access_token {
      println!("access_token has been updated");
      OAuthStorage::_save_token_auth(token);
    }
    else {
      println!("ah shit");
    }
    self.user_access_token = token.to_owned();

    Ok(())
  }
}