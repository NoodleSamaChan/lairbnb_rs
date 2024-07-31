use actix_web::http::header::ContentType;
use actix_web::HttpResponse;
use actix_web_flash_messages::IncomingFlashMessages;
use std::fmt::Write;

pub async fn insert_lair_form(
    flash_messages: IncomingFlashMessages,
) -> Result<HttpResponse, actix_web::Error> {
    let mut msg_html = String::new();
    for m in flash_messages.iter() {
        writeln!(msg_html, "<p><i>{}</i></p>", m.content()).unwrap();
    }

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta http-equiv="content-type" content="text/html; charset=utf-8">
    <title>Add new lair</title>
</head>
<body>
    {msg_html}
    <form action="/admin/dashboard/insert_lair" method="post">
        <label>Title:<br>
            <input
                type="text"
                placeholder="Enter the lair name"
                name="title"
            >
        </label>
        <br>
        <label>Description:<br>
            <textarea
                placeholder="Enter the lair description"
                name="description"
                rows="20"
                cols="50"
            ></textarea>
        </label>
        <br>
        <label>Lair image:<br>
            <textarea
                placeholder="Enter the lair image"
                name="image"
                rows="20"
                cols="50"
            ></textarea>
        </label>
        <br>
        <label>Lair longitude:<br>
            <textarea
                placeholder="Enter the lair longitude"
                name="lon"
                rows="20"
                cols="50"
            ></textarea>
        </label>
        <br>
        <label>Lair latitude:<br>
            <textarea
                placeholder="Enter the lair latitude"
                name="lat"
                rows="20"
                cols="50"
            ></textarea>
        </label>
        <br>
        <button type="submit">Submit lair</button>
    </form>
    <p><a href="/admin/dashboard">&lt;- Back</a></p>
</body>
</html>"#,
        )))
}
