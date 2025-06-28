#[macro_use] extern crate rocket;

use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use rocket::serde::json::Json;
use rocket::State;
use serde::Serialize;
use opencv::{
    prelude::*,
    videoio,
    core,
    imgproc,
};



#[derive(Serialize, Clone)]
struct ScoreData {
    camera_id: i32,
    ball_count: u32,
}

type SharedScores = Arc<Mutex<Vec<ScoreData>>>;
type ActiveFlag = Arc<AtomicBool>;

#[get("/scores")]
fn get_scores(scores: &State<SharedScores>) -> Json<Vec<ScoreData>> {
    let scores = scores.lock().unwrap();
    Json(scores.clone())
}

#[post("/toggle_tracking")]
fn toggle_tracking(active: &State<ActiveFlag>) -> Json<bool> {
    let currently_active = active.load(Ordering::SeqCst);
    let new_state = !currently_active;
    active.store(new_state, Ordering::SeqCst);
    Json(new_state)
}

#[launch]
fn rocket() -> _ {
    let initial_scores = vec![
        ScoreData { camera_id: 0, ball_count: 0 },
        ScoreData { camera_id: 1, ball_count: 0 },
    ];


    let shared_scores = Arc::new(Mutex::new(initial_scores));
    let active_flag = Arc::new(AtomicBool::new(false)); // Initially inactive

    // Clone for thread
    let scores_clone = Arc::clone(&shared_scores);
    let active_clone = Arc::clone(&active_flag);

    std::thread::spawn(move || {
        multi_camera_capture(scores_clone, active_clone);
    });

    rocket::build()
        .manage(shared_scores)
        .manage(active_flag)
        .mount("/api", routes![get_scores, toggle_tracking])
        .mount("/", rocket::fs::FileServer::from("static"))
        .configure(rocket::Config {
            address: "0.0.0.0".parse().unwrap(),
            port: 8000,
            ..rocket::Config::default()
        })
}

fn multi_camera_capture(shared_scores: SharedScores, active: ActiveFlag) {
    // Try camera IDs 0 and 1; you can extend this if you want
    let camera_ids = vec![0, 1];

    let mut captures: Vec<(i32, videoio::VideoCapture)> = vec![];

    for &id in &camera_ids {
        let cap = videoio::VideoCapture::new(id, videoio::CAP_ANY).unwrap();
        if cap.is_opened().unwrap() {
            captures.push((id, cap));
            println!("Opened camera {}", id);
        } else {
            eprintln!("Warning: Unable to open camera {}, skipping", id);
        }
    }

    if captures.is_empty() {
        panic!("No cameras available to capture");
    }

    loop {
        if active.load(Ordering::SeqCst) {
            for (camera_id, cap) in &mut captures {
                let mut frame = Mat::default();
                if cap.read(&mut frame).unwrap() && !frame.empty() {
                    let ball_count = detect_objects(&frame);

                    let mut scores = shared_scores.lock().unwrap();
                    if let Some(score) = scores.iter_mut().find(|s| s.camera_id == *camera_id) {
                        score.ball_count = ball_count;
                    }
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}


fn detect_objects(frame: &Mat) -> u32 {
    let mut gray = Mat::default();
    imgproc::cvt_color(
        frame,
        &mut gray,
        imgproc::COLOR_BGR2GRAY,
        0,
        core::AlgorithmHint::ALGO_HINT_DEFAULT
    ).unwrap();

    let mut thresh = Mat::default();
    imgproc::threshold(&gray, &mut thresh, 200.0, 255.0, imgproc::THRESH_BINARY).unwrap();

    let mut contours = core::Vector::<core::Vector<core::Point>>::new();
    imgproc::find_contours(
        &thresh,
        &mut contours,
        imgproc::RETR_EXTERNAL,
        imgproc::CHAIN_APPROX_SIMPLE,
        core::Point::new(0, 0),
    ).unwrap();

    contours.len() as u32
}
