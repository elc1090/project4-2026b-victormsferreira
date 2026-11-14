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

#[derive(Serialize, Deserialize, PartialEq, Clone, ImplicitClone)]
struct Test {
    text: String,
}

#[autoprops]
#[component]
fn TestList(records: &Vec<Record<Test>>) -> Html {
    html! {
        for record in records {
            <p> {&record.item.text} </p>
        }
    }
}

#[autoprops]
#[component]
fn TestFetcher(pb: Rc<RefCell<Pocketbase>>) -> HtmlResult {
    let tests = use_future(|| async move { pb.borrow().full_list::<Test>("test", None).await })?;
    let records = &*tests;
    Ok(html! {
        <TestList records={records.clone()}/>
    })
}

#[autoprops]
#[component]
fn App(pb: Rc<RefCell<Pocketbase>>) -> Html {
    html! {
        <>
        <Suspense fallback={html! {<p> {"Loading..."} </p>}} >
            <TestFetcher pb={pb}/>
        </Suspense>
        </>
    }
}

#[derive(Clone)]
struct LoginInfo {
    username: String,
    password: String,
}

#[autoprops]
#[component]
fn LoginScreen(pb: Rc<RefCell<Pocketbase>>) -> Html {
    let login_info = use_state(|| LoginInfo {
        username: "".to_string(),
        password: "".to_string(),
    });
    let login: UseAsyncHandle<(), ()> = {
        let login_info = login_info.clone();
        use_async(async move {
            Ok({pb
                .borrow_mut()
                .auth_with_password("users", &login_info.username, &login_info.password)
                .await;
            log!(&pb.borrow().auth_store.token);
            ()
            })
        })
    };
    let onsubmit = {
        let login_info = login_info.clone();
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
        <form {onsubmit}>
        <input oninput={on_usernameinput} type="text" />
        <input oninput={on_passwordinput} type="password" />
        <button type="submit">{"Login"} </button>
        </form>
    }
}

fn main() {
    let pocketbase = Rc::new(RefCell::new(Pocketbase::new("http://127.0.0.1:8090")));
    yew::Renderer::<LoginScreen>::with_props(LoginScreenProps { pb: pocketbase }).render();
}
