use crate::Route;
use crate::pb::Pocketbase;
use gloo_console::log;
use gloo_render::AnimationFrame;
use gloo_render::request_animation_frame;
use hecs::*;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
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
struct GameLevel {
    name: String,
    author: String,
    data: String,
}

#[repr(usize)]
enum Input {
    Left,
    Right,
    Up,
    Down,
}

struct TilePos {
    x: i32,
    y: i32,
}

struct Sprite {
    id: i32,
    prio: u8,
    render_pos: (f64, f64),
}

struct Wall {}
struct Player {}
struct Moving {}
struct Box {}
struct Target {}

struct GameState {
    img: Option<HtmlImageElement>,
    canvas: Option<Canvas>,
    level: Option<GameLevel>,
    world: hecs::World,
    level_size: (u32, u32),
    level_complete: bool,
    cam_pos: (i32, i32),
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}
impl GameState {
    pub fn restart(&mut self) {
        self.world.clear();
        self.parse_level();
        self.level_complete = false;
    }
    fn move_objects(&mut self, input: Input) {
        let mut to_move: Vec<(Entity, &Input)> = Vec::new();

        let moving: HashMap<(i32, i32), Entity> = self
            .world
            .query::<(Entity, &TilePos, &Moving)>()
            .iter()
            .map(|t| ((t.1.x, t.1.y), t.0))
            .collect::<HashMap<_, _>>();
        let walls: HashMap<(i32, i32), Entity> = self
            .world
            .query::<(Entity, &TilePos, &Wall)>()
            .iter()
            .map(|t| ((t.1.x, t.1.y), t.0))
            .collect::<HashMap<_, _>>();

        for (position, _player) in self.world.query::<(&TilePos, &Player)>().iter() {
            let (start, end, is_x) = match &input {
                Input::Up => (position.y, 0, false),
                Input::Down => (position.y, (self.level_size.1 - 1) as i32, false),
                Input::Left => (position.x, 0, true),
                Input::Right => (position.x, (self.level_size.0 - 1) as i32, true),
            };

            let range = if start < end {
                (start..=end).collect::<Vec<_>>()
            } else {
                (end..=start).rev().collect::<Vec<_>>()
            };
            for p in range {
                let pos = if is_x {
                    (p, position.y)
                } else {
                    (position.x, p)
                };
                match moving.get(&pos) {
                    Some(b) => {
                        to_move.push((*b, &input));
                    }
                    None => match walls.get(&pos) {
                        Some(_) => {
                            to_move.clear();
                        }
                        None => break,
                    },
                }
            }
        }
        for (e, input) in to_move {
            let mut position = self.world.get::<&mut TilePos>(e).unwrap();
            match input {
                Input::Up => position.y -= 1,
                Input::Down => position.y += 1,
                Input::Left => position.x -= 1,
                Input::Right => position.x += 1,
            }
        }
    }

    fn check_for_solution(&mut self) -> bool {
        let boxes: Vec<(i32, i32)> = self
            .world
            .query::<(Entity, &TilePos, &Box)>()
            .iter()
            .map(|t| (t.1.x, t.1.y))
            .collect::<Vec<_>>();

        for (position, _target) in self.world.query::<(&TilePos, &Target)>().iter() {
            let mut satisfied = false;
            for box_pos in &boxes {
                if box_pos.0 == position.x && box_pos.1 == position.y {
                    satisfied = true;
                    break;
                }
            }
            if !satisfied {
                return false;
            }
        }
        true
    }
    pub fn on_input_down(&mut self, input: Input) {
        if !self.level_complete {
            self.move_objects(input);
            self.level_complete = self.check_for_solution();
        }
    }

    pub fn spawn_box(&mut self, x: i32, y: i32) {
        self.world.spawn((
            TilePos { x, y },
            Sprite {
                id: 3,
                prio: 128,
                render_pos: (x as f64 * 16.0, y as f64 * 16.0),
            },
            Moving {},
            Box {},
        ));
    }
    pub fn spawn_grass(&mut self, x: i32, y: i32) {
        self.world.spawn((
            TilePos { x, y },
            Sprite {
                id: 0,
                prio: 0,
                render_pos: (x as f64 * 16.0, y as f64 * 16.0),
            },
        ));
    }
    pub fn spawn_wall(&mut self, x: i32, y: i32) {
        self.world.spawn((
            TilePos { x, y },
            Sprite {
                id: 1,
                prio: 1,
                render_pos: (x as f64 * 16.0, y as f64 * 16.0),
            },
            Wall {},
        ));
    }
    pub fn spawn_player(&mut self, x: i32, y: i32) {
        self.world.spawn((
            TilePos { x, y },
            Sprite {
                id: 2,
                prio: 255,
                render_pos: (x as f64 * 16.0, y as f64 * 16.0),
            },
            Player {},
            Moving {},
        ));
    }
    pub fn spawn_target(&mut self, x: i32, y: i32) {
        self.world.spawn((
            TilePos { x, y },
            Sprite {
                id: 4,
                prio: 64,
                render_pos: (x as f64 * 16.0, y as f64 * 16.0),
            },
            Target {},
        ));
    }

    pub fn parse_level(&mut self) {
        if let Some(level) = &self.level.clone() {
            let mut w = 0;
            let mut x = 0;
            let mut y = 0;
            for i in level.data.chars() {
                match i {
                    '.' => self.spawn_grass(x, y),
                    '#' => self.spawn_wall(x, y),
                    '@' => {
                        self.spawn_grass(x, y);
                        self.spawn_player(x, y);
                    }
                    'B' => {
                        self.spawn_grass(x, y);
                        self.spawn_box(x, y);
                    }
                    'T' => {
                        self.spawn_grass(x, y);
                        self.spawn_target(x, y);
                    }
                    '\n' => {
                        w = w.max(x);
                        x = -1;
                        y += 1;
                    }
                    _ => {}
                }

                x += 1;
            }
            self.level_size = (w as u32, (y + 1) as u32);
            self.cam_pos.0 = (self.level_size.0 * 8) as i32;
            self.cam_pos.1 = (self.level_size.1 * 8) as i32;
        }
    }
    pub fn on_draw(&mut self) {
        self.clear();
        let mut draw_queue = vec![];
        for (position, tile) in self.world.query_mut::<(&TilePos, &mut Sprite)>() {
            tile.render_pos.0 = lerp(tile.render_pos.0, (position.x * 16) as f64, 0.25);
            tile.render_pos.1 = lerp(tile.render_pos.1, (position.y * 16) as f64, 0.25);
            draw_queue.push((
                tile.id,
                tile.render_pos.0 as i32,
                tile.render_pos.1 as i32,
                tile.prio,
            ));
        }
        draw_queue.sort_by_key(|v| v.3);
        for v in draw_queue {
            self.sprite(v.0, v.1, v.2);
        }
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
            let dx = (x - self.cam_pos.0 + 160) as f64;
            let dy = (y - self.cam_pos.1 + 120) as f64;
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
        _t_a: f64,
        t_b: f64,
        frame_handler: Rc<RefCell<Option<AnimationFrame>>>,
    ) {
        game_state.borrow_mut().on_draw();
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
        world: World::new(),
        level_size: (0, 0),
        cam_pos: (0, 0),
        level_complete: false,
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
                game_state.borrow_mut().parse_level();
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
                    "ArrowUp" => game_state.borrow_mut().on_input_down(Input::Up),
                    "ArrowDown" => game_state.borrow_mut().on_input_down(Input::Down),
                    "ArrowLeft" => game_state.borrow_mut().on_input_down(Input::Left),
                    "ArrowRight" => game_state.borrow_mut().on_input_down(Input::Right),
                    "r" => game_state.borrow_mut().restart(),
                    _ => (),
                }
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
        <canvas tabindex="0" {onkeydown} id="gameCanvas" width="320" height="240" ref={canvas_ref} />
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
