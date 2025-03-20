// import Rocket
#[macro_use] extern crate rocket;

// this is our get route which will be requested at the "/" location wherever it is mounted
#[get("/")]
fn say_hello() -> &'static str {
  "Hello, welcome to the api!"
}

// start the web server and mount our get route at "/api". Can replace /api with anything
// or just leave it as "/" as the default location
#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/api", routes![say_hello])
        .configure(rocket::Config {
            address: "0.0.0.0".parse().unwrap(),  // Bind to all interfaces
            port: 8000,  // Set the port to 8000
            ..rocket::Config::default()  // Use default settings for other configurations
        })
}
