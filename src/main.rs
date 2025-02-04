mod db;
use db::{Db, Job};
use rocket::{form::Form, fs::TempFile, http::Status};
use rocket_db_pools::{Connection, Database};

#[macro_use]
extern crate rocket;

#[derive(FromForm)]
struct Upload<'r> {
    callback_api_url: String,
    file: TempFile<'r>,
}

#[get("/")]
fn healthcheck() -> &'static str {
    "ok"
}

#[post("/generate", data = "<upload>")]
async fn generate(db: Connection<Db>, mut upload: Form<Upload<'_>>) -> Status {
    let filename = match upload.file.name() {
        Some(v) => v,
        None => "tmp1",
    };

    let filetype = upload.file.content_type().unwrap().extension().unwrap();
    let fp = format!("tracks/{}.{}", filename, filetype);

    match upload.file.persist_to(&fp).await {
        Ok(_) => {}
        Err(e) => {
            println!("{e}");

            return Status::BadRequest;
        }
    };

    let mut job = Job::new(fp, upload.callback_api_url.clone());

    match job.persist(db).await {
        Ok(_) => {}
        Err(_) => {
            return Status::BadRequest;
        }
    };

    return Status::Ok;
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Db::init())
        .mount("/", routes![healthcheck, generate])
}
