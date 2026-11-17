#[allow(dead_code)]
mod pb;
use crate::pb::{Pocketbase};
use std::rc::Rc;
use yew::{Callback, prelude::*};
use yew_autoprops::autoprops;
use yew_router::prelude::*;
mod auth;
use auth::{LoginForm, RedirectOAuth, SignUpForm};

#[autoprops]
#[component]
fn Home(pb: Rc<Pocketbase>, on_login: Callback<Pocketbase>, username: AttrValue) -> Html {
    html! {
        <div style="display: flexbox; width: 100%; height: 100%; flex-direction: column">
        <SignUpForm pb={Rc::clone(&pb)} />
        <LoginForm pb={Rc::clone(&pb)} on_login_event={on_login} />
        <p>{username}</p>
        </div>
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
    let switch = {
        let on_login = {
            let pb = pb.clone();
            let username = username.clone();
            Callback::from(move |new_pb: Pocketbase| {
                pb.set(Rc::new(new_pb));
                let un = if let Some(authstore) = Rc::clone(&pb).auth_store.clone() {
                    authstore.record.name.clone()
                } else {
                    None
                };
                username.set(un);
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
                    html! {<Home pb={Rc::clone(&pb)} on_login={on_login.clone()} username={username}/>}
                }
                Route::RedirectOAuth => {
                    html! {<RedirectOAuth pb={Rc::clone(&pb)} on_login={on_login.clone()}/>}
                }
            }
        })
    };
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch}/>
        </BrowserRouter>
    }
}

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[at("/oauth/redirect")]
    RedirectOAuth,
}

fn main() {
    yew::Renderer::<App>::new().render();
}
