use crate::game;
use crate::pb::Pocketbase;
use gloo_console::log;
use gloo_render::AnimationFrame;
use gloo_render::request_animation_frame;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::window;
use web_sys::{CanvasRenderingContext2d as Canvas, HtmlCanvasElement};
use yew::{Callback, prelude::*};
use yew_autoprops::autoprops;
use yew_router::prelude::*;

struct GameState {
    canvas: Option<Canvas>,
    counter: i32,
    timer: f64,
}

impl GameState {
    fn game_tick(
        game_state: Rc<RefCell<GameState>>,
        t_a: f64,
        t_b: f64,
        frame_handler: Rc<RefCell<Option<AnimationFrame>>>,
    ) {
        let dt = (t_b - t_a) / 1000.0;
        game_state.borrow_mut().timer += dt;
        game_state.borrow_mut().counter += 1;

        if let Some(canvas) = &game_state.borrow().canvas {
            let r = (f64::abs(f64::sin(game_state.borrow().timer)) * 255.0) as i32;
            let style = format!("rgb({} 0 0)", r);
            canvas.set_fill_style_str(&style);
            canvas.fill_rect(0.0, 0.0, 320.0, 240.0);
        }
        let cb = {
            let frame_handler = frame_handler.clone();
            move |t| {
                let t_a = t_b;
                let t_b = t;
                GameState::game_tick(game_state.clone(), t_a, t_b, frame_handler);
            }
        };
        *frame_handler.borrow_mut() = Some(request_animation_frame(cb));
    }
}

#[autoprops]
#[component]
pub fn Game(pb: Rc<Pocketbase>) -> Html {
    let canvas_ref = use_node_ref();
    let game_state = use_mut_ref(|| GameState {
        canvas: None,
        counter: 0,
        timer: 0.0,
    });
    let frame_handler: Rc<RefCell<Option<AnimationFrame>>> = use_mut_ref(|| None);
    {
        let canvas_ref = canvas_ref.clone();
        let game_state = game_state.clone();
        let frame_handler = frame_handler.clone();
        use_effect_with(canvas_ref, move |canvas_ref| {
            let canvas = canvas_ref
                .cast::<HtmlCanvasElement>()
                .expect("canvas_ref not attached to canvas element")
                .get_context("2d")
                .expect("could not get canvas context")
                .unwrap()
                .dyn_into::<Canvas>()
                .unwrap();

            game_state.borrow_mut().canvas = Some(canvas);

            let cb = {
                let frame_handler = frame_handler.clone();
                move |t| {
                    let t_a = 0.0;
                    let t_b = t;
                    GameState::game_tick(game_state.clone(), t_a, t_b, frame_handler);
                }
            };

            *frame_handler.borrow_mut() = Some(request_animation_frame(cb));
        });
    }

    html! {
        <canvas width="320" height="240" ref={canvas_ref} />
    }
}
