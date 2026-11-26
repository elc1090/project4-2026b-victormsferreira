mod auth;
mod editor;
mod game;
mod header;
mod home;
#[allow(dead_code)]
mod pb;

use crate::{editor::Editor, pb::Pocketbase};
use auth::{LoginForm, RedirectOAuth, SignUpForm};
use game::Game;
use gloo_console::log;
use header::Header;
use home::{Home, UserPage};
use std::{env, rc::Rc};
use web_sys::window;
use yew::{Callback, prelude::*};
use yew_autoprops::autoprops;
use yew_router::prelude::*;

#[autoprops]
#[component]
fn App() -> Html {
    let addr = env::var("POCKETBASE_URL").unwrap();
    let pb: UseStateHandle<Rc<Pocketbase>> =
        use_state(|| Rc::new(Pocketbase::new(&addr)));
    let navigator = use_navigator().unwrap();
    let on_logout = {
        let pb = pb.clone();
        let navigator = navigator.clone();
        Callback::from(move |_: ()| {
            pb.set(Rc::new(pb.logout()));
            navigator.push(&Route::Home);
            log!("Logged Out");
        })
    };

    let switch = {
        let on_login = {
            let pb = pb.clone();
            let navigator = navigator.clone();
            Callback::from(move |new_pb: Pocketbase| {
                if new_pb.auth_store.is_some() {
                    pb.set(Rc::new(new_pb));
                    navigator.push(&Route::Home);
                } else {
                    window()
                        .unwrap()
                        .alert_with_message("Login inválido.")
                        .unwrap();
                    navigator.push(&Route::Login);
                }
            })
        };

        let pb = pb.clone();

        Callback::from(move |route| match route {
            Route::Home => {
                html! {<Home pb={Rc::clone(&pb)}/>}
            }
            Route::Login => {
                html! { <LoginForm pb={Rc::clone(&pb)} on_login_event={on_login.clone()} />}
            }
            Route::SignUp => {
                html! { <SignUpForm pb={Rc::clone(&pb)} />}
            }
            Route::RedirectOAuth => {
                html! {<RedirectOAuth pb={Rc::clone(&pb)} on_login_event={on_login.clone()}/>}
            }
            Route::User { id } => {
                html! {<UserPage pb={Rc::clone(&pb)} id={id} />}
            }
            Route::Game { level } => {
                html! {<Game pb={Rc::clone(&pb)} level_id={level}/>}
            }
            Route::Edit => {
                html! {<Editor pb={Rc::clone(&pb)} level_id={None}/>}
            }
            Route::EditLevel { level } => {
                html! {<Editor pb={Rc::clone(&pb)} level_id={Some(level)}/>}
            }
        })
    };
    html! {
        <>
        <Header pb={Rc::clone(&pb)} on_logout={on_logout} />
        <Switch<Route>  render={switch}/>
        </>
    }
}

#[autoprops]
#[component]
fn Router() -> Html {
    html! {
        <BrowserRouter>

            <App/>
        </BrowserRouter>
    }
}

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[at("/login")]
    Login,
    #[at("/signUp")]
    SignUp,

    #[at("/oauth/redirect")]
    RedirectOAuth,

    #[at("/user/:id")]
    User { id: String },

    #[at("/play/:level")]
    Game { level: String },

    #[at("/edit")]
    Edit,

    #[at("/edit/:level")]
    EditLevel { level: String },
}

fn main() {
    yew::Renderer::<Router>::new().render();
}
