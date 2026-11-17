#[allow(dead_code)]
mod pb;
use crate::pb::Pocketbase;
use std::rc::Rc;
use yew::{Callback, prelude::*};
use yew_autoprops::autoprops;
use yew_router::prelude::*;
mod auth;
use auth::{LoginForm, SignUpForm};

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
    let on_loginform_login = {
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

    html! {
        <div style="display: flexbox; width: 100%; height: 100%; flex-direction: column">
        <SignUpForm pb={Rc::clone(&*pb)} />
        <LoginForm pb={Rc::clone(&*pb)} on_login_event={on_loginform_login} />
        <p>{if let Some(name) = &*username {name} else {"No User"}}</p>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
