use crate::pb::Pocketbase;
use gloo_console::log;
use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use web_sys::{HtmlElement, HtmlInputElement};
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
fn ProviderLogo(logo: AttrValue) -> Html {
    let node_ref = use_node_ref();
    {
        let node_ref = node_ref.clone();
        let logo = logo.clone();
        use_effect_with(node_ref, move |node_ref| {
            let div = node_ref
                .cast::<HtmlElement>()
                .expect("div_ref not attached to div element");
            div.set_inner_html(&logo);
        });
    }

    html! {
        <div ref={node_ref}></div>
    }
}

#[autoprops]
#[component]
fn OAuth2Login(pb: Rc<Pocketbase>) -> HtmlResult {
    let auth_methods = {
        let pb = Rc::clone(&pb);
        use_future(|| async move { pb.list_auth_methods("users").await })
    };
    const REDIRECT_URL: &str = "http://127.0.0.1:8090/redirect";
    match &*&auth_methods {
        Ok(auths) => Ok(html! {
            <div>
            for provider in &auths.oauth2.providers {
                <a href={format!("{}{}", provider.auth_url, REDIRECT_URL)} onclick={
                    let name = provider.name.clone();
                    let verifier = provider.code_verifier.clone();
                    Callback::from(move |_| {
                        let _ = LocalStorage::set("ProviderName", &name);
                        let _ = LocalStorage::set("CodeVerifier", &verifier);
                        log!(&name);
                })}>
                <ProviderLogo logo={provider.logo.clone()} />
                <p>{&provider.display_name}</p>
                 </a>
            }
            </div>
        }),
        Err(_) => Ok(html! {<p>{"Erro carregando Metódos de Autenticação"}</p>}),
    }
}

#[autoprops]
#[component]
pub fn LoginForm(pb: Rc<Pocketbase>, on_login_event: Callback<Pocketbase>) -> Html {
    let login_info = use_state(|| LoginInfo {
        username: "".to_string(),
        password: "".to_string(),
    });
    let login: UseAsyncHandle<(), ()> = {
        let login_info = login_info.clone();
        let pb = pb.clone();
        use_async(async move {
            Ok(on_login_event.emit(
                pb.auth_with_password("users", &login_info.username, &login_info.password)
                    .await,
            ))
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
        <form {onsubmit} style="display: flexbox; width: 100%; height: 100%; flex-direction: column;">
            <label for={"username"}><b>{"Nome de Usuário"}</b></label>
            <input autocomplete={"username"} id={"username"} name={"username"} oninput={on_usernameinput} type="text" />
            <label for={"password"}><b>{"Senha"}</b></label>
            <input id={"password"} name={"password"} oninput={on_passwordinput} type="password" />
            <button type={"submit"}>{"Login"} </button>
            <Suspense fallback={html! {<p> {"..."} </p>}} >
                <OAuth2Login pb={pb.clone()} />
            </Suspense>
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
pub fn SignUpForm(pb: Rc<Pocketbase>) -> Html {
    let signup_info = use_state(|| SignUpInfo {
        ..Default::default()
    });
    let login: UseAsyncHandle<(), ()> = {
        let signup_info = signup_info.clone();
        let pb = pb.clone();
        use_async(async move {
            Ok({
                let _ = pb.create("users", &*(signup_info)).await;
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
        let signup_info: UseStateHandle<SignUpInfo> = signup_info.clone();
        Callback::from(move |e: InputEvent| {
            let mut c = (*signup_info).clone();
            c.password_confirm = e.target_unchecked_into::<HtmlInputElement>().value();
            signup_info.set(c);
        })
    };
    html! {
        <form {onsubmit} style="display: flexbox; width: 100%; height: 100%; flex-direction: column;">
        <label for={"email"}><b>{"Email"}</b></label>
        <input autocomplete={"username"} id={"email"} name={"email"} oninput={on_emailinput} type="text" />
        <label for={"username"}><b>{"Nome de Usuário"}</b></label>
        <input autocomplete={"username"} id={"username"} name={"username"} oninput={on_usernameinput} type="text" />
        <label for={"password"}><b>{"Senha"}</b></label>
        <input autocomplete={"new-password"} id={"password"} name={"password"} oninput={on_passwordinput} type="password" />
        <label for={"password_confirm"}><b>{"Confirme sua senha"}</b></label>
        <input autocomplete={"new-password"} id={"password_confirm"} name={"password_confirm"} oninput={on_passwordconfirminput} type="password" />
        <button type="submit">{"Registrar"} </button>
        </form>
    }
}
