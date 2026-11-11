#[allow(dead_code)]
mod pb;
use crate::pb::{Pocketbase, Record};
use implicit_clone::ImplicitClone;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use yew::{prelude::*, suspense::use_future};
use yew_autoprops::autoprops;

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
fn TestFetcher(pb: Arc<Pocketbase>) -> HtmlResult {
    let tests = use_future(|| async move { pb.full_list::<Test>("test", None).await })?;
    let records = &*tests;
    Ok(html! {
        <TestList records={records.clone()}/>
    })
}

#[autoprops]
#[component]
fn App(pb: Arc<Pocketbase>) -> Html {
    html! {
        <>
        <Suspense fallback={html! {<p> {"Loading..."} </p>}} >
            <TestFetcher pb={pb}/>
        </Suspense>
        </>
    }
}

fn main() {
    let pocketbase = Arc::new(Pocketbase::new("http://127.0.0.1:8090"));
    yew::Renderer::<App>::with_props(AppProps { pb: pocketbase}).render();
}
