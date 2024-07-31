use actix_web::{http::header::ContentType, HttpResponse};
use actix_web_flash_messages::IncomingFlashMessages;
use std::fmt::Write;

pub async fn sign_up_form(flash_messages: IncomingFlashMessages) -> HttpResponse {
    let mut error_html = String::new();
    for m in flash_messages.iter() {
        writeln!(error_html, "<p><i>{}</i></p>", m.content()).unwrap();
    }
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta http-equiv="content-type" content="text/html; charset=utf-8">
    <title>Sign up</title>
</head>
<body>
    {error_html}
    <form action="/registration" method="post">
        <label>Username
            <input
                type="text"
                placeholder="Enter Username"
                name="name"
            >
        </label>
        <label>Email
            <input
                type="text"
                placeholder="Enter email"
                name="email"
            >
        </label>
        <label>Password
            <input
                type="password"
                placeholder="Enter Password"
                name="password"
            >
        </label>
        <button type="submit">Register</button>
    </form>
</body>
</html>"#,
        ))
}
