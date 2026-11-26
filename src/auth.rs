use crate::Route;
use crate::pb::{OAuth2, Pocketbase};
use gloo_console::log;
use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use std::env;
use std::rc::Rc;
use web_sys::{HtmlInputElement, window};
use yew::{Callback, prelude::*, suspense::use_future};
use yew_autoprops::autoprops;
use yew_hooks::*;
use yew_router::hooks::{use_location, use_navigator};
use crate::REDIRECT_URL;
#[derive(Clone)]
struct LoginInfo {
    username: String,
    password: String,
}

#[autoprops]
#[component]
fn OAuth2Login(pb: Rc<Pocketbase>) -> HtmlResult {
    let auth_methods = {
        let pb = Rc::clone(&pb);
        use_future(|| async move { pb.list_auth_methods("users").await })
    };
    

    match &auth_methods {
        Ok(auths) => Ok(html! {
            for provider in &auths.oauth2.providers {
                <button type="button" onclick={
                    let name = provider.name.clone();
                    let verifier = provider.code_verifier.clone();
                    let url = format!("{}{}", provider.auth_url, REDIRECT_URL);
                    Callback::from(move |_| {
                        LocalStorage::set("ProviderName", &name).unwrap();
                        LocalStorage::set("CodeVerifier", &verifier).unwrap();
                        let window = window().unwrap();
                        log!(&url);
                        window.document().unwrap().location().unwrap().set_href(&url).unwrap();
                })}>
                {&provider.display_name}
                 </button>
            }
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
        let on_login_event = on_login_event.clone();
        use_async(async move {
            on_login_event.emit(
                pb.auth_with_password("users", &login_info.username, &login_info.password)
                    .await,
            );
            Ok(())
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
        <div class="centerContainer">
        <form {onsubmit} class="authForm">
            <h3>{"Login"}</h3>
            <input placeholder="Email" autocomplete={"username"} id={"username"} name={"username"} oninput={on_usernameinput} type="text" />
            <input placeholder="Senha" id={"password"} name={"password"} oninput={on_passwordinput} type="password" />
            <button type={"submit"}>{"Login"} </button>
            <Suspense fallback={html! {<p> {"..."} </p>}} >
                <OAuth2Login pb={pb.clone()}/>
            </Suspense>
        </form>
        </div>
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SignUpInfo {
    email: String,
    name: String,
    password: String,
    password_confirm: String,
}

#[autoprops]
#[component]
pub fn SignUpForm(pb: Rc<Pocketbase>) -> Html {
    let signup_info = use_state(|| SignUpInfo {
        ..Default::default()
    });
    let navigator = use_navigator().unwrap();
    let login: UseAsyncHandle<(), ()> = {
        let signup_info = signup_info.clone();
        let pb = pb.clone();
        use_async(async move {
            let _ = pb.create("users", &*(signup_info)).await;
            navigator.push(&Route::Login);
            Ok(())
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
            c.name = e.target_unchecked_into::<HtmlInputElement>().value();
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
        <div class="centerContainer">
        <form {onsubmit} class="authForm">
            <h3>{"Registro"}</h3>
            <input placeholder="Email" autocomplete={"username"} id={"email"} name={"email"} oninput={on_emailinput} type="text" />
            <input placeholder="Nome de Usuário" autocomplete={"username"} id={"username"} name={"username"} oninput={on_usernameinput} type="text" />
            <input placeholder="Senha" autocomplete={"new-password"} id={"password"} name={"password"} oninput={on_passwordinput} type="password" />
            <input placeholder="Confirme sua Senha" autocomplete={"new-password"} id={"password_confirm"} name={"password_confirm"} oninput={on_passwordconfirminput} type="password" />
            <button type="submit">{"Registrar"} </button>
        </form>
        </div>
    }
}

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RedirectOAuthParams {
    code: String,
}

#[autoprops]
#[component]
pub fn RedirectOAuth(pb: Rc<Pocketbase>, on_login_event: Callback<Pocketbase>) -> Html {
    let location = use_location().unwrap();
    let provider = location.query::<RedirectOAuthParams>().unwrap();
    let auth = use_future(|| async move {
        log!(&provider.code);
        let provider_name: String = LocalStorage::get::<String>("ProviderName").unwrap();
        let provider_code: String = provider.code;
        let verifier = LocalStorage::get::<String>("CodeVerifier").unwrap();
        let oauth: OAuth2<()> = OAuth2::<()> {
            provider: provider_name.clone(),
            code: provider_code.clone(),
            code_verifier: verifier.clone(),
            redirect_URL: REDIRECT_URL.to_string(),
            create_data: None,
        };
        let pb = pb.auth_with_oauth2("users", &oauth).await;
        on_login_event.emit(pb);
        LocalStorage::set("ProviderCode", ()).unwrap();
    });

    match auth {
        Ok(_) => html! {<p>{"Redirecting..."}</p>},
        Err(_) => html! {<p>{"Redirecting..."}</p>},
    }
}
