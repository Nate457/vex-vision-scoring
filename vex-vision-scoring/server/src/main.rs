#[macro_use] extern crate rocket;

use std::path::PathBuf;
use rocket::fs::{FileServer};

// API route
#[get("/")]
fn say_hello() -> &'static str {
    "Hello, welcome to the API!"
}

fn static_files_path() -> PathBuf {
    let mut dir = std::env::current_dir().unwrap();
    dir.push("static");
    dir
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/api", routes![say_hello])
        // Serve static files (index.html, pkg/, etc) at root "/"
         .mount("/", FileServer::from(static_files_path()))
        .configure(rocket::Config {
            address: "0.0.0.0".parse().unwrap(),
            port: 8000,
            ..rocket::Config::default()
        })
}

