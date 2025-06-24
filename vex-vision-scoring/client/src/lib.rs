use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{window, HtmlButtonElement, HtmlDivElement};
use wasm_bindgen::JsCast;
use serde::Deserialize;
use serde_wasm_bindgen;

#[wasm_bindgen(start)]
pub fn start() {
    let window = window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");

    let toggle_btn = document.get_element_by_id("toggleBtn")
        .expect("should have toggleBtn")
        .dyn_into::<HtmlButtonElement>()
        .expect("toggleBtn should be a button");

    let score_display = document.get_element_by_id("scoreDisplay")
        .expect("should have scoreDisplay")
        .dyn_into::<HtmlDivElement>()
        .expect("scoreDisplay should be a div");

    toggle_btn.set_text_content(Some("Start Tracking"));

    let tracking_active = std::rc::Rc::new(std::cell::Cell::new(false));
    let tracking_active_for_closure = tracking_active.clone();
    let score_display_for_closure = score_display.clone();
    let toggle_btn_for_closure = toggle_btn.clone();
    let window_clone = window.clone();

    let onclick_closure = Closure::wrap(Box::new(move || {
        let tracking_active_inner = tracking_active_for_closure.clone();
        let score_display_inner = score_display_for_closure.clone();
        let toggle_btn_inner = toggle_btn_for_closure.clone();
        let window_inner = window_clone.clone();

        spawn_local(async move {
            let mut opts = web_sys::RequestInit::new();
            opts.set_method("POST");

            let fetch_promise = window_inner.fetch_with_str_and_init("/api/toggle_tracking", &opts);
            let resp = JsFuture::from(fetch_promise).await.unwrap();
            let response: web_sys::Response = resp.dyn_into().unwrap();

            let json_promise = response.json().unwrap();
            let json = JsFuture::from(json_promise).await.unwrap();

            let is_active = json.as_bool().unwrap_or(false);
            tracking_active_inner.set(is_active);

            toggle_btn_inner.set_text_content(Some(
                if is_active { "Stop Tracking" } else { "Start Tracking" }
            ));

            if is_active {
                poll_scores(score_display_inner, tracking_active_inner);
            }
        });
    }) as Box<dyn Fn()>);

    toggle_btn.set_onclick(Some(onclick_closure.as_ref().unchecked_ref()));
    onclick_closure.forget();
}

fn poll_scores(score_display: HtmlDivElement, tracking_active: std::rc::Rc<std::cell::Cell<bool>>) {
    let win = window().unwrap();

    let closure = Closure::wrap(Box::new(move || {
        if !tracking_active.get() {
            return;
        }

        let score_display = score_display.clone();

        spawn_local(async move {
            let window = window().unwrap();

            let fetch_promise = window.fetch_with_str("/api/scores");

            let resp_js = match JsFuture::from(fetch_promise).await {
                Ok(resp) => resp,
                Err(_) => {
                    score_display.set_text_content(Some("Error fetching scores"));
                    return;
                }
            };

            let response: web_sys::Response = resp_js.dyn_into().unwrap();

            let json_promise = response.json().unwrap();
            let json_jsvalue = match JsFuture::from(json_promise).await {
                Ok(val) => val,
                Err(_) => {
                    score_display.set_text_content(Some("Error awaiting JSON"));
                    return;
                }
            };

            let scores: Vec<ScoreData> = match serde_wasm_bindgen::from_value(json_jsvalue) {
                Ok(scores) => scores,
                Err(_) => {
                    score_display.set_text_content(Some("Error deserializing scores"));
                    return;
                }
            };

            let html = scores.iter()
                .map(|s| format!("Camera {}: {} balls", s.camera_id, s.ball_count))
                .collect::<Vec<_>>()
                .join("<br>");

            score_display.set_inner_html(&html);
        });
    }) as Box<dyn Fn()>);

    win.set_interval_with_callback_and_timeout_and_arguments_0(
        closure.as_ref().unchecked_ref(),
        1000,
    ).expect("should register interval");

    closure.forget();
}

#[derive(Deserialize, Default)]
struct ScoreData {
    camera_id: i32,
    ball_count: u32,
}
