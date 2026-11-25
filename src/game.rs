use crate::Route;
use crate::pb::Pocketbase;
use gloo_console::log;
use gloo_render::AnimationFrame;
use gloo_render::request_animation_frame;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;
use web_sys::window;
use web_sys::{CanvasRenderingContext2d as Canvas, HtmlCanvasElement, HtmlImageElement};
use yew::{prelude::*, suspense::use_future};
use yew_autoprops::autoprops;
use yew_router::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct LevelData {
    tiles: Vec<u32>,
    player_spawn: (i32, i32),
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct GameLevel {
    name: String,
    author: String,
    width: u32,
    height: u32,
    data: LevelData,
}

trait GameObject {
    fn update(&mut self, dt: f64);

    fn draw(&self, game_state: &GameState);
}

#[repr(usize)]
enum Input {
    Left,
    Right,
    Up,
    Down,
    Max,
}

#[derive(Default, Clone)]
struct InputSystem {
    inputs_prev: [bool; Input::Max as usize],
    inputs: [bool; Input::Max as usize],
    mouse: (i32, i32),
}

impl InputSystem {
    fn update_prev(&mut self) {
        self.inputs_prev = self.inputs;
    }
    fn input_down(&self, input: Input) -> bool {
        self.inputs[input as usize]
    }
    fn input_pressed(&self, input: Input) -> bool {
        let input = input as usize;
        self.inputs[input] && !self.inputs_prev[input]
    }
    fn input_released(&self, input: Input) -> bool {
        let input = input as usize;
        !self.inputs[input] && self.inputs_prev[input]
    }

    fn input_axis(&self, neg: Input, pos: Input) -> f64 {
        (if self.inputs[pos as usize] { 1.0 } else { 0.0 })
            - (if self.inputs[neg as usize] { 1.0 } else { 0.0 })
    }
    pub fn on_input_down(&mut self, input: Input) {
        self.inputs[input as usize] = true;
    }
    pub fn on_input_up(&mut self, input: Input) {
        self.inputs[input as usize] = false;
    }
}

#[derive(Default)]
enum PlayerAnim {
    #[default]
    Idle,
    Walk {
        left: bool,
        frame: i32,
    },
    Jump {
        left: bool,
    },
}
#[derive(Default)]
struct Player {
    input_state: InputSystem,
    x: f64,
    y: f64,
    vel_x: f64,
    vel_y: f64,
    anim: PlayerAnim,
    flip: bool,
    anim_time: f64,
    jump_buffer: f64,
    coyote_time: f64,
}
impl GameObject for Player {
    fn update(&mut self, dt: f64) {
        const GRAVITY: f64 = 1024.0;
        const MAX_SPEED: f64 = 150.0;
        const ACCEL: f64 = 1024.0;
        const DECCEL: f64 = 0.85;
        const JUMP_SPEED: f64 = 250.0;
        const JUMP_GRAVITY: f64 = 512.0;
        let accel = self.input_state.input_axis(Input::Left, Input::Right);
        self.anim_time += dt;
        self.x += self.vel_x * dt / 2.0;
        self.y += self.vel_y * dt / 2.0;
        if self.jump_buffer > 0.0 {
            self.jump_buffer -= dt;
        }

        if self.input_state.input_pressed(Input::Up) {
            self.jump_buffer = 0.2;
        }

        if accel != 0.0 {
            self.vel_x += ACCEL * accel * dt;
            if self.vel_x < -MAX_SPEED {
                self.vel_x = -MAX_SPEED;
            } else if self.vel_x > MAX_SPEED {
                self.vel_x = MAX_SPEED;
            }
            self.flip = self.vel_x < 0.0;
        } else {
            self.vel_x *= DECCEL;
        }
        if self.y < 240.0 {
            if self.vel_y < 0.0 && self.input_state.input_down(Input::Up) {
                self.vel_y += JUMP_GRAVITY * dt;
            } else {
                self.vel_y += GRAVITY * dt;
            }
        } else {
            self.y = 240.0;
            self.coyote_time = 0.15;
            if self.vel_y > 0.0 {
                self.vel_y = 0.0;
            }
        }
        if self.vel_y < 0.0 && self.input_state.input_released(Input::Up) {
            self.vel_y *= 0.5;
        }
        if self.coyote_time > 0.0 && self.jump_buffer > 0.0 {
            self.vel_y = -JUMP_SPEED;
            self.jump_buffer = -1.0;
            self.coyote_time = -1.0;
        }

        self.anim = if self.vel_y < 0.0 {
            PlayerAnim::Jump { left: self.flip }
        } else if self.vel_x < -10.0 {
            PlayerAnim::Walk {
                left: true,
                frame: (self.anim_time * 10.0) as i32,
            }
        } else if self.vel_x > 10.0 {
            PlayerAnim::Walk {
                left: false,
                frame: (self.anim_time * 10.0) as i32,
            }
        } else {
            PlayerAnim::Idle
        };

        self.x += self.vel_x * dt / 2.0;
        self.y += self.vel_y * dt / 2.0;
    }
    fn draw(&self, game_state: &GameState) {
        let spr = match self.anim {
            PlayerAnim::Idle => 280,
            PlayerAnim::Jump { left } => {
                if left {
                    304
                } else {
                    284
                }
            }
            PlayerAnim::Walk { left, frame } => {
                if left {
                    301 + frame % 3
                } else {
                    281 + frame % 3
                }
            }
        };
        game_state.sprite(spr, self.x as i32 + 8, self.y as i32 - 16);
    }
}

struct GameState {
    img: Option<HtmlImageElement>,
    canvas: Option<Canvas>,
    level: Option<GameLevel>,
    player: Player,
    input: InputSystem,
}

impl GameState {
    pub fn on_update(&mut self, dt: f64) {
        self.player.input_state = self.input.clone();
        self.player.update(dt);
        self.input.update_prev();
    }
    pub fn on_draw(&self) {
        self.clear();
        self.player.draw(self);
    }
    pub fn sprite(&self, spr: i32, x: i32, y: i32) {
        if let Some(img) = &self.img
            && let Some(canvas) = &self.canvas
        {
            const TILE_SIZE: i32 = 16;
            let sx = (TILE_SIZE * (spr % 20)) as f64;
            let sy = (TILE_SIZE * (spr / 20)) as f64;
            let sw = TILE_SIZE as f64;
            let sh = TILE_SIZE as f64;
            let dx = x as f64;
            let dy = y as f64;
            let dw = TILE_SIZE as f64;
            let dh = TILE_SIZE as f64;

            let res = canvas
                .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                    img, sx, sy, sw, sh, dx, dy, dw, dh,
                );
            if let Err(res) = res {
                log!(res);
            }
        }
    }
    pub fn clear(&self) {
        if let Some(canvas) = &self.canvas {
            canvas.set_fill_style_str("rgb(0, 0, 0)");
            canvas.fill_rect(0.0, 0.0, 320.0, 240.0);
            canvas.set_fill_style_str("rgb(255, 255, 255)");
        }
    }
    pub fn tile(&self, spr: i32, x: i32, y: i32) {
        const TILE_SIZE: i32 = 16;
        self.sprite(spr, x * TILE_SIZE, y * TILE_SIZE);
    }
    async fn load_level_by_id(pb: Rc<Pocketbase>, level_id: &str) -> Option<GameLevel> {
        if let Ok(level_record) = pb.view::<GameLevel>("levels", level_id).await {
            log!(&level_record.item.name);
            Some(level_record.item)
        } else {
            None
        }
    }
    fn game_tick(
        game_state: Rc<RefCell<GameState>>,
        t_a: f64,
        t_b: f64,
        frame_handler: Rc<RefCell<Option<AnimationFrame>>>,
    ) {
        let dt = (t_b - t_a) / 1000.0;
        game_state.borrow_mut().on_update(dt);
        game_state.borrow().on_draw();
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

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
struct User {
    email: String,
    email_visibility: bool,
    verified: bool,
    name: String,
}

#[autoprops]
#[component]
pub fn Game(pb: Rc<Pocketbase>, level_id: AttrValue) -> Html {
    let canvas_ref = use_node_ref();
    let img_ref = use_node_ref();
    let navigator = use_navigator().unwrap();
    let game_state = use_mut_ref(|| GameState {
        img: None,
        canvas: None,
        level: None,
        player: Default::default(),
        input: Default::default(),
    });
    let frame_handler: Rc<RefCell<Option<AnimationFrame>>> = use_mut_ref(|| None);
    let author_name = {
        let game_state = game_state.clone();
        let pb = pb.clone();
        use_future(|| async move {
            let level = GameState::load_level_by_id(pb.clone(), &level_id).await;
            if let Some(level) = level {
                let author = pb.view::<User>("users", &level.author).await;
                game_state.borrow_mut().level = Some(level);
                Some(author.unwrap().item.name.clone())
            } else {
                navigator.push(&Route::Home);
                None
            }
        })
    };

    {
        let img_ref = img_ref.clone();
        let game_state = game_state.clone();
        use_effect_with(img_ref, move |img_ref| {
            let img = img_ref
                .cast::<HtmlImageElement>()
                .expect("canvas_ref not attached to canvas element");
            game_state.borrow_mut().img = Some(img);
        });
    }

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
            canvas.set_image_smoothing_enabled(false);
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
    let onkeydown = {
        let game_state = game_state.clone();
        Callback::from(move |e: KeyboardEvent| {
            e.prevent_default();
            if !e.repeat() {
                match e.key().as_str() {
                    "w" => game_state.borrow_mut().input.on_input_down(Input::Up),
                    "s" => game_state.borrow_mut().input.on_input_down(Input::Down),
                    "a" => game_state.borrow_mut().input.on_input_down(Input::Left),
                    "d" => game_state.borrow_mut().input.on_input_down(Input::Right),
                    _ => (),
                }
            }
        })
    };
    let onkeyup = {
        let game_state = game_state.clone();
        Callback::from(move |e: KeyboardEvent| {
            e.prevent_default();
            if !e.repeat() {
                match e.key().as_str() {
                    "w" => game_state.borrow_mut().input.on_input_up(Input::Up),
                    "s" => game_state.borrow_mut().input.on_input_up(Input::Down),
                    "a" => game_state.borrow_mut().input.on_input_up(Input::Left),
                    "d" => game_state.borrow_mut().input.on_input_up(Input::Right),
                    _ => (),
                }
            }
        })
    };
    let onmousemove = {
        let game_state = game_state.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            let x = e.x();
            let y = e.y();
            let target = e.target().unwrap().dyn_into::<HtmlCanvasElement>().ok();
            if let Some(target) = target {
                let dom_rect = target.get_bounding_client_rect();
                let x = x as f64 - dom_rect.x();
                let y = y as f64 - dom_rect.y();
                let xr = x / dom_rect.width();
                let yr = y / dom_rect.height();
                let xo = (xr * 320.0) as i32;
                let yo = (yr * 240.0) as i32;
                game_state.borrow_mut().input.mouse = (xo, yo);
            }
        })
    };
    use_effect(|| {
        let _ = window()
            .unwrap()
            .document()
            .unwrap()
            .query_selector("#gameCanvas")
            .unwrap()
            .unwrap()
            .dyn_ref::<HtmlElement>()
            .unwrap()
            .focus();
    });

    html! {
        <>
        <div class="centerContainer" style="margin: 2%;">
        <canvas tabindex="0" {onkeydown} {onkeyup} {onmousemove} id="gameCanvas" width="320" height="240" ref={canvas_ref} />
        if let Ok(author_name) = author_name {
            <div style="display: flex; gap: 0px; flex-direction: column; justify-content: center; align-items: center">
            <h1 style="margin: 16px;">{game_state.borrow().level.clone().unwrap().name}</h1>
            if let Some(author_name) = &*author_name {
                <h3 style="margin: 0px;">{format!("Por {}", author_name)}</h3>
            }
            </div>
        } else {
            <h1 class="loadingText">{"Loading"}</h1>
        }
        </div>
        <img style="display: none;" src="/img/monochrome_tilemap_packed.png" ref={img_ref}/>
        </>
    }
}
