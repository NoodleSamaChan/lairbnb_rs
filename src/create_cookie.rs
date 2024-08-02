use crate::{
    authentication::Credentials,
    domain::{SubscriberName, SubscriberPassword},
};
use actix_web::{http::Error, web, HttpRequest, HttpResponse};
use base64::{engine::general_purpose::URL_SAFE, Engine};
use reqwest::{cookie, StatusCode};
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use sqlx::PgPool;

pub fn create_cookie(username: &SubscriberName, password: &SubscriberPassword) -> String {
    let secret = String::from("airbnb");

    let cookie = format!("{}:{}", username.as_ref(), password.as_ref());

    let final_cookie: Vec<u8> = cookie
        .bytes()
        .zip(secret.bytes().cycle())
        .map(|(cookie, secret)| cookie ^ secret)
        .collect();

    URL_SAFE.encode(final_cookie)
}

#[derive(Deserialize)]
pub struct UserInfo {
    pub name: String,
    pub password: String,
}

pub fn extract_cookie(request: HttpRequest) -> Result<UserInfo, anyhow::Error> {
    let header = request.headers();
    let cookie = header.get("Authorization");
    let cookie = cookie.unwrap().to_str()?;
    let mut cookie = cookie.split(" ");
    let cookie = cookie.nth(1).unwrap();

    let decoded_image_data = URL_SAFE.decode(cookie).unwrap();

    let secret = String::from("airbnb");

    let final_cookie: Vec<u8> = decoded_image_data
        .iter()
        .zip(secret.bytes().cycle())
        .map(|(decoded_image_data, secret)| decoded_image_data ^ secret)
        .collect();

    let info: String = String::from_utf8(final_cookie).unwrap();
    println!("{}", info);
    let mut info = info.split(":");
    Ok(UserInfo {
        name: String::from(info.nth(0).unwrap()),
        password: String::from(info.nth(0).unwrap()),
    })
}
