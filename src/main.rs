use rocket::{form::Form, fs::TempFile};

#[macro_use]
extern crate rocket;

#[derive(FromForm)]
struct Upload<'r> {
    file: TempFile<'r>,
}

#[get("/")]
fn healthcheck() -> &'static str {
    "ok"
}

#[post("/generate", data = "<upload>")]
async fn generate(mut upload: Form<Upload<'_>>) -> Result<(), std::io::Error> {
    let filename = match upload.file.name() {
        Some(v) => v,
        None => "tmp1",
    };

    let filetype = upload.file.content_type().unwrap().extension().unwrap();
    let fp = format!("tracks/{}.{}", filename, filetype);

    upload.file.persist_to(fp).await
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![healthcheck, generate])
}
