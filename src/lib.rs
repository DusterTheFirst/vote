use std::collections::HashMap;

use askama::Template;
use hmac::{Hmac, NewMac};
use jwt::SignWithKey;
use oauth2::{
    basic::BasicClient, http::StatusCode, reqwest::async_http_client, AuthUrl, AuthorizationCode,
    ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use response::Response;
use sha2::Sha256;
use templates::FormTemplate;
use twilight_model::{
    id::GuildId,
    user::{CurrentUser, CurrentUserGuild},
};
use url::Url;
use utils::set_panic_hook;
use wasm_bindgen::prelude::*;

mod response;
mod templates;
mod utils;

mod env_vars {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        pub static client_secret: String;
        pub static client_id: String;
        pub static guild_id: String;
        pub static secret_key: String;
        pub static redirect_url: String;
        pub static form_embed_url: String;
    }
}

#[wasm_bindgen]
pub async fn handle_request(request: web_sys::Request) -> web_sys::Response {
    set_panic_hook();

    let url = Url::parse(&request.url()).expect("Invalid url");

    let client_id = ClientId::new(env_vars::client_id.to_string());
    let client_secret = Some(ClientSecret::new(env_vars::client_secret.to_string()));

    let auth_url = AuthUrl::new("https://discord.com/api/oauth2/authorize".to_string())
        .expect("Invalid auth url");
    let token_url = Some(
        TokenUrl::new("https://discord.com/api/oauth2/token".to_string())
            .expect("Invalid token url"),
    );

    let oauth_client = BasicClient::new(client_id, client_secret, auth_url, token_url)
        .set_redirect_url(
            RedirectUrl::new(env_vars::redirect_url.to_string()).expect("Invalid redirect url"),
        );

    handle_request_internal(url, oauth_client).await.into()
}

pub async fn handle_request_internal(url: Url, oauth: BasicClient) -> Response {
    match url.path() {
        "/" => Response::Redirect(
            oauth
                .authorize_url(|| CsrfToken::new(":)".to_string()))
                .add_scope(Scope::new("identify".to_string()))
                .add_scope(Scope::new("email".to_string()))
                .add_scope(Scope::new("guilds".to_string()))
                .url()
                .0
                .to_string(),
        ),
        "/oauth/authorize" => {
            let query: HashMap<_, _> = url.query_pairs().collect();

            let token_result = match oauth
                .exchange_code(AuthorizationCode::new(
                    query
                        .get("code")
                        .expect("Missing code query parameter")
                        .to_string(),
                ))
                .request_async(async_http_client)
                .await
            {
                Ok(result) => result,
                Err(error) => {
                    console_err!("Error processing user's token: {:?}", error);

                    return Response::Redirect("/".to_string());
                }
            };

            let token = token_result.access_token().secret();

            let client = reqwest::Client::new();

            let guilds: Vec<CurrentUserGuild> = client
                .get("https://discord.com/api/v8/users/@me/guilds")
                .header("Authorization", format!("Bearer {}", token))
                .send()
                .await
                .expect("Failed to get current user")
                .error_for_status()
                .expect("Received error from discord")
                .json()
                .await
                .expect("Failed to parse current user");

            let guild_id = GuildId(env_vars::guild_id.parse().unwrap());
            if !guilds.iter().any(|x| x.id == guild_id) {
                return Response::Content(
                    "You are not in the guild that this poll takes place in".to_string(),
                );
            }

            let user: CurrentUser = client
                .get("https://discord.com/api/v8/users/@me")
                .header("Authorization", format!("Bearer {}", token))
                .send()
                .await
                .expect("Failed to get current user")
                .error_for_status()
                .expect("Received error from discord")
                .json()
                .await
                .expect("Failed to parse current user");

            let token = user
                .clone()
                .sign_with_key(
                    &Hmac::<Sha256>::new_varkey(
                        &base64::decode(env_vars::secret_key.as_str())
                            .expect("Malformed secret_key"),
                    )
                    .unwrap(),
                )
                .expect("Failed to sign jwt");

            Response::Content(
                // reqwest::get(env_vars::form_embed_url.as_str()).await.unwrap().text().await.unwrap()
                FormTemplate {
                    username: user.name,
                    discriminator: user.discriminator,
                    token,
                    email: user.email.unwrap_or_else(|| "<no email found>".to_string()),
                    form_embed_url: env_vars::form_embed_url.to_string(),
                }
                .render()
                .unwrap(),
            )
        }
        _ => StatusCode::NOT_IMPLEMENTED.into(),
    }
}
