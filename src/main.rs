#[allow(dead_code)]
mod pb;
use crate::pb::{Pocketbase, Record};
use gloo_console::log;
use implicit_clone::ImplicitClone;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use web_sys::HtmlInputElement;
use yew::{Callback, prelude::*, suspense::use_future};
use yew_autoprops::autoprops;
use yew_hooks::*;

#[derive(Clone)]
struct LoginInfo {
    username: String,
    password: String,
}

#[autoprops]
#[component]
fn LoginForm(pb: Rc<RefCell<Pocketbase>>) -> Html {
    let login_info = use_state(|| LoginInfo {
        username: "".to_string(),
        password: "".to_string(),
    });
    let login: UseAsyncHandle<(), ()> = {
        let login_info = login_info.clone();
        use_async(async move {
            Ok({
                pb.borrow_mut()
                    .auth_with_password("users", &login_info.username, &login_info.password)
                    .await;
                log!(&pb.borrow().auth_store.token);
                ()
            })
        })
    };
    let onsubmit = {
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            login.run();
        })
    };
    let on_usernameinput: Callback<InputEvent> = {
        let login_info = login_info.clone();
        Callback::from(move |e: InputEvent| {
            let mut c = (*login_info).clone();
            c.username = e.target_unchecked_into::<HtmlInputElement>().value();
            login_info.set(c);
        })
    };
    let on_passwordinput: Callback<InputEvent> = {
        let login_info = login_info.clone();
        Callback::from(move |e: InputEvent| {
            let mut c = (*login_info).clone();
            c.password = e.target_unchecked_into::<HtmlInputElement>().value();
            login_info.set(c);
        })
    };
    html! {
        <form {onsubmit} style="display: flexbox; width: 100%; height: 100%; flex-direction: row    ">
        <label for={"username"}><b>{"Nome de Usuário"}</b></label>
        <input autocomplete={"username"} id={"username"} name={"username"} oninput={on_usernameinput} type="text" />
        <label for={"password"}><b>{"Senha"}</b></label>
        <input id={"password"} name={"password"} oninput={on_passwordinput} type="password" />
        <button type="submit">{"Login"} </button>
        </form>
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SignUpInfo {
    email: String,
    username: String,
    password: String,
    password_confirm: String,
}

#[autoprops]
#[component]
fn SignUpForm(pb: Rc<RefCell<Pocketbase>>) -> Html {
    let signup_info = use_state(|| SignUpInfo {
        ..Default::default()
    });
    let login: UseAsyncHandle<(), ()> = {
        let signup_info = signup_info.clone();
        use_async(async move {
            Ok({
                let _ = pb.borrow().create("users", &*(signup_info)).await;
                ()
            })
        })
    };
    let onsubmit = {
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            login.run();
        })
    };
    let on_usernameinput: Callback<InputEvent> = {
        let signup_info = signup_info.clone();
        Callback::from(move |e: InputEvent| {
            let mut c = (*signup_info).clone();
            c.username = e.target_unchecked_into::<HtmlInputElement>().value();
            signup_info.set(c);
        })
    };
    let on_emailinput: Callback<InputEvent> = {
        let signup_info = signup_info.clone();
        Callback::from(move |e: InputEvent| {
            let mut c = (*signup_info).clone();
            c.email = e.target_unchecked_into::<HtmlInputElement>().value();
            signup_info.set(c);
        })
    };

    let on_passwordinput: Callback<InputEvent> = {
        let signup_info = signup_info.clone();
        Callback::from(move |e: InputEvent| {
            let mut c = (*signup_info).clone();
            c.password = e.target_unchecked_into::<HtmlInputElement>().value();
            signup_info.set(c);
        })
    };
    let on_passwordconfirminput: Callback<InputEvent> = {
        let signup_info = signup_info.clone();
        Callback::from(move |e: InputEvent| {
            let mut c = (*signup_info).clone();
            c.password_confirm = e.target_unchecked_into::<HtmlInputElement>().value();
            signup_info.set(c);
        })
    };
    html! {
        <form {onsubmit}>
        <label for={"email"}><b>{"Email"}</b></label>
        <input id={"email"} name={"email"} oninput={on_emailinput} type="text" />
        <label for={"username"}><b>{"Nome de Usuário"}</b></label>
        <input id={"username"} name={"username"} oninput={on_usernameinput} type="text" />
        <label for={"password"}><b>{"Senha"}</b></label>
        <input id={"password"} name={"password"} oninput={on_passwordinput} type="password" />
        <label for={"password_confirm"}><b>{"Confirme sua senha"}</b></label>
        <input id={"password_confirm"} name={"password_confirm"} oninput={on_passwordconfirminput} type="password" />
        <button type="submit">{"Registrar"} </button>
        </form>
    }
}

#[autoprops]
#[component]
fn App(pb: Rc<RefCell<Pocketbase>>) -> Html {
    html! {
        <div style="display: flexbox; width: 100%; height: 100%; flex-direction: column">
        <SignUpForm pb={pb.clone()}/>
        <LoginForm pb={pb.clone()}/>
        </div>
    }
}

fn main() {
    let pocketbase = Rc::new(RefCell::new(Pocketbase::new("http://127.0.0.1:8090")));
    yew::Renderer::<App>::with_props(AppProps { pb: pocketbase }).render();
}
