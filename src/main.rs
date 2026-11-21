#[allow(dead_code)]
mod pb;
use crate::{pb::Pocketbase};
use std::rc::Rc;
use web_sys::window;
use yew::{Callback, prelude::*};
use yew_autoprops::autoprops;
use yew_router::{prelude::*};
mod auth;
use auth::{LoginForm, RedirectOAuth, SignUpForm};
use gloo_console::log;

#[autoprops]
#[component]
fn Home(pb: Rc<Pocketbase>, username: AttrValue) -> Html {
    html! {
        //<div style="display: flexbox; width: 100%; height: 100%; flex-direction: column">
        //<SignUpForm pb={Rc::clone(&pb)} />
       <h3>{username}</h3>
        //</div>
    }
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
    let switch = {
        let on_login = {
            let pb = pb.clone();
            let username = username.clone();
            Callback::from(move |new_pb: Pocketbase| {
                log!(&pb.addr);
                log!("Got OnLogin Callback");
                if new_pb.auth_store.is_some() {
                    pb.set(Rc::new(new_pb));
                    let un = if let Some(authstore) = Rc::clone(&pb).auth_store.clone() {
                        authstore.record.name.clone()
                    } else {
                        None
                    };
                    username.set(un);
                    navigator.push(&Route::Home);
                    log!("Testing Testing 123");
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

            }
        })
    };
    html! {
        <Switch<Route> render={switch}/>
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
}



fn main() {
    yew::Renderer::<Router>::new().render();
}
