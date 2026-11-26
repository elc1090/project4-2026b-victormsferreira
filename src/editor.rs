use crate::Route;
use crate::game::GameLevel;
use crate::pb::Pocketbase;
use gloo_console::log;
use gloo_render::AnimationFrame;
use gloo_render::request_animation_frame;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;
use web_sys::window;
use web_sys::{
    CanvasRenderingContext2d as Canvas, HtmlCanvasElement, HtmlImageElement, HtmlInputElement,
};
use yew::{prelude::*, suspense::use_future};
use yew_autoprops::autoprops;
use yew_hooks::*;
use yew_router::prelude::*;

struct EditorState {
    img: Option<HtmlImageElement>,
    name: String,
    id: Option<String>,
    canvas: Option<Canvas>,
    tiles: [[char; 15]; 20],
    current_tile: char,
    mouse_pos: (i32, i32),
}
impl EditorState {
    pub fn build_data(&self) -> String {
        let mut str = String::new();
        for i in 0..15 {
            for j in 0..20 {
                str.push(self.tiles[j][i]);
            }
            str.push('\n');
        }
        str
    }
    pub fn parse_data(data: String) -> [[char; 15]; 20] {
        let mut x = 0;
        let mut y = 0;
        let mut tiles = [[' '; 15]; 20];
        for i in data.chars() {
            if i == '\n' {
                x = -1;
                y += 1;
            } else {
                tiles[x as usize][y as usize] = i;
            }
            x += 1;
        }
        tiles
    }

    pub fn on_mouse_move(&mut self, x: i32, y: i32, left: bool, right: bool) {
        self.mouse_pos = (x / 16, y / 16);
        if self.mouse_pos.0 >= 0
            && self.mouse_pos.0 < 20
            && self.mouse_pos.1 >= 0
            && self.mouse_pos.1 < 15
            && left
        {
            if right {
                self.tiles[self.mouse_pos.0 as usize][self.mouse_pos.1 as usize] = ' ';
            } else {
                self.tiles[self.mouse_pos.0 as usize][self.mouse_pos.1 as usize] =
                    self.current_tile;
            }
        }
    }

    pub fn on_draw(&mut self) {
        self.clear();
        if let Some(canvas) = self.canvas.clone() {
            canvas.set_fill_style_str("rgb(255, 255, 255)");
            canvas.set_stroke_style_str("rgb(255, 255, 255)");
            for i in (0..320).step_by(16) {
                for j in (0..240).step_by(16) {
                    canvas.stroke_rect(i as f64, j as f64, 16.0, 16.0);
                }
            }
        }
        for i in 0..20 {
            for j in 0..15 {
                if let Some(tile) = EditorState::char_to_tile(self.tiles[i][j]) {
                    self.tile(tile, i as i32, j as i32);
                }
            }
        }
        if let Some(tile) = EditorState::char_to_tile(self.current_tile) {
            self.tile(tile, self.mouse_pos.0, self.mouse_pos.1);
        }
    }
    fn char_to_tile(c: char) -> Option<i32> {
        match c {
            '.' => Some(0),
            '#' => Some(1),
            '@' => Some(2),
            'B' => Some(3),
            'T' => Some(4),
            _ => None,
        }
    }
    pub fn sprite(&self, spr: i32, x: i32, y: i32) {
        if let Some(img) = &self.img
            && let Some(canvas) = &self.canvas
        {
            const TILE_SIZE: i32 = 16;
            let sx = 0.0;
            let sy = (TILE_SIZE * spr) as f64;
            let sw = TILE_SIZE as f64;
            let sh = TILE_SIZE as f64;
            let dx = (x) as f64;
            let dy = (y) as f64;
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
    pub fn tile(&self, spr: i32, x: i32, y: i32) {
        const TILE_SIZE: i32 = 16;
        self.sprite(spr, x * TILE_SIZE, y * TILE_SIZE);
    }
    pub fn clear(&self) {
        if let Some(canvas) = &self.canvas {
            canvas.set_fill_style_str("rgb(0, 0, 0)");
            canvas.fill_rect(0.0, 0.0, 320.0, 240.0);
            canvas.set_fill_style_str("rgb(255, 255, 255)");
        }
    }
    fn editor_tick(
        state: Rc<RefCell<EditorState>>,
        _t_a: f64,
        t_b: f64,
        frame_handler: Rc<RefCell<Option<AnimationFrame>>>,
    ) {
        state.borrow_mut().on_draw();
        let cb = {
            let frame_handler = frame_handler.clone();
            move |t| {
                let t_a = t_b;
                let t_b = t;
                EditorState::editor_tick(state.clone(), t_a, t_b, frame_handler);
            }
        };
        *frame_handler.borrow_mut() = Some(request_animation_frame(cb));
    }
}

#[autoprops]
#[component]
pub fn Editor(pb: Rc<Pocketbase>, level_id: Option<AttrValue>) -> Html {
    let name_input_ref = use_node_ref();
    let canvas_ref = use_node_ref();
    let img_ref = use_node_ref();
    let navigator = use_navigator().unwrap();
    let editor_state = use_mut_ref(|| EditorState {
        img: None,
        canvas: None,
        name: "Unnamed".to_string(),
        id: if let Some(id) = level_id.clone() {
            Some(id.to_string())
        } else {
            None
        },
        tiles: [[' '; 15]; 20],
        current_tile: '.',
        mouse_pos: (0, 0),
    });
    let frame_handler: Rc<RefCell<Option<AnimationFrame>>> = use_mut_ref(|| None);
    let _ = {
        let pb = pb.clone();
        let level_id = level_id.clone();
        let editor_state = editor_state.clone();
        let navigator = navigator.clone();
        let name_input_ref = name_input_ref.clone();
        use_future(|| async move {
            if let Some(id) = level_id {
                let level = pb.view::<GameLevel>("levels", &id).await;
                if let Ok(level) = level {
                    editor_state.borrow_mut().tiles = EditorState::parse_data(level.item.data);
                    editor_state.borrow_mut().name = level.item.name.clone();
                    name_input_ref
                        .cast::<HtmlInputElement>()
                        .expect("name_input_ref not attached to input")
                        .set_value(&editor_state.borrow().name);
                    Some(level.item.name.clone())
                } else {
                    navigator.push(&Route::Home);
                    None
                }
            } else {
                None
            }
        })
    };
    {
        let img_ref = img_ref.clone();
        let editor_state = editor_state.clone();
        use_effect_with(img_ref, move |img_ref| {
            let img = img_ref
                .cast::<HtmlImageElement>()
                .expect("canvas_ref not attached to canvas element");
            editor_state.borrow_mut().img = Some(img);
        });
    }

    {
        let canvas_ref = canvas_ref.clone();
        let editor_state = editor_state.clone();
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
            editor_state.borrow_mut().canvas = Some(canvas);

            let cb = {
                let frame_handler = frame_handler.clone();
                move |t| {
                    let t_a = 0.0;
                    let t_b = t;
                    EditorState::editor_tick(editor_state.clone(), t_a, t_b, frame_handler);
                }
            };

            *frame_handler.borrow_mut() = Some(request_animation_frame(cb));
        });
    }

    let img_onclick = {
        let editor_state = editor_state.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            let y = e.y();
            let target = e.target().unwrap().dyn_into::<HtmlImageElement>().ok();
            if let Some(target) = target {
                let dom_rect = target.get_bounding_client_rect();
                let y = y as f64 - dom_rect.y();
                let yr = y / dom_rect.height();
                let ys = (yr * 5.0) as i32;

                editor_state.borrow_mut().current_tile = match ys {
                    0 => '.',
                    1 => '#',
                    2 => '@',
                    3 => 'B',
                    4 => 'T',
                    _ => '.',
                };
            }
        })
    };
    let onmousemove = {
        let editor_state = editor_state.clone();
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
                editor_state.borrow_mut().on_mouse_move(
                    xo,
                    yo,
                    e.buttons() & 1 != 0,
                    e.shift_key(),
                );
            }
        })
    };
    let on_save = {
        let navigator = navigator.clone();
        let pb = pb.clone();
        let editor_state = editor_state.clone();
        let name = editor_state.borrow().name.clone();
        let data = editor_state.borrow().build_data();
        let id = editor_state.borrow().id.clone();
        use_async(async move {
            let level = GameLevel {
                name,
                author: pb.auth_store.clone().unwrap().record.id,
                data,
                public: true,
            };
            if let Some(id) = id {
                pb.update("levels", &id, &level).await.unwrap();
                Ok(())
            } else {
                if let Ok(record) = pb.create("levels", &level).await {
                    navigator.push(&Route::EditLevel {
                        level: record.id.clone(),
                    });
                    Ok(())
                } else {
                    Err(())
                }
            }
        })
    };
    let onkeydown: Callback<KeyboardEvent> = {
        let on_save = on_save.clone();
        Callback::from(move |e: KeyboardEvent| {
            e.prevent_default();
            if e.key() == "s" && e.ctrl_key() {
                on_save.run();
            }
        })
    };

    let save = {
        Callback::from(move |_: MouseEvent| {
            on_save.run();
        })
    };

    let on_nameinput: Callback<InputEvent> = {
        let editor_state = editor_state.clone();
        Callback::from(move |e: InputEvent| {
            editor_state.borrow_mut().name = e.target_unchecked_into::<HtmlInputElement>().value();
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
        <div style="display: flex; flex-direction: row; gap: 10%; align-items: center; image-rendering: pixelated;">
            <img onclick={img_onclick} style="width: 20%; scale:6.0; border:2px solid;" src="/img/monochrome_tilemap_packed.png" />
            <canvas onmousemove={onmousemove.clone()} onmousedown={onmousemove.clone()} tabindex="0" {onkeydown} id="gameCanvas" width="320" height="240" ref={canvas_ref} />
            <div style="width: 20%;">
                <h2>{"Clique para adicionar items"} </h2>
                <h2>{"Shift+Clique para remover items"} </h2>
            </div>
        </div>
        <div style="display: flex; gap: 0px; flex-direction: column; justify-content: center; align-items: center">
        <input oninput={on_nameinput} type="text" style="margin: 16px;" ref={name_input_ref}/>
        <button onclick={save}>{"Salvar"}</button>
        </div>
        </div>
        <img style="display: none;" src="/img/monochrome_tilemap_packed.png" ref={img_ref}/>
        </>
    }
}
