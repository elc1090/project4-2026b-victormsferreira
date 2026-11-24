#[allow(dead_code)]
mod pb;
use crate::pb::Pocketbase;
use std::rc::Rc;
use web_sys::window;
use yew::{Callback, prelude::*};
use yew_autoprops::autoprops;
use yew_router::prelude::*;
mod auth;
use auth::{LoginForm, RedirectOAuth, SignUpForm};
mod header;
use gloo_console::log;
mod game;
use game::Game;

use header::Header;
#[autoprops]
#[component]
fn Home(pb: Rc<Pocketbase>, username: AttrValue) -> Html {
    html! {}
}

#[autoprops]
#[component]
fn App() -> Html {
    let pb: UseStateHandle<Rc<Pocketbase>> =
        use_state(|| Rc::new(Pocketbase::new("http://127.0.0.1:8090")));
    let username: UseStateHandle<Option<String>> = use_state(|| {
        if let Some(authstore) = Rc::clone(&pb).auth_store.clone() {
            authstore.record.name.clone()
        } else {
            None
        }
    });
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

        Callback::from(move |route| {
            let us = username.clone();
            let username = if let Some(u) = &*us {
                u.clone()
            } else {
                "No User".to_string()
            };
            match route {
                Route::Home => {
                    html! {<Home pb={Rc::clone(&pb)} username={username}/>}
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
                    html! {<p> {id}</p>}
                }
                Route::Game => {
                    html! {<Game pb={Rc::clone(&pb)}/>}
                }
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
    #[at("/Login")]
    Login,
    #[at("/SignUp")]
    SignUp,

    #[at("/oauth/redirect")]
    RedirectOAuth,

    #[at("/user/:id")]
    User { id: String },

    #[at("/game")]
    Game,
}

fn main() {
    yew::Renderer::<Router>::new().render();
}
