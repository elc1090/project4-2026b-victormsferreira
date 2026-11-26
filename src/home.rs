use crate::game::{GameLevel, User};
use crate::pb::ListParams;
use crate::{
    Route,
    pb::{Pocketbase, Record},
};
use std::rc::Rc;
use yew::{Callback, prelude::*, suspense::use_future};
use yew_autoprops::autoprops;
use yew_router::prelude::*;

#[autoprops]
#[component]
pub fn Level(pb: Rc<Pocketbase>, level: &Record<GameLevel>) -> Html {
    let navigator = use_navigator().unwrap();
    let onclick = {
        let navigator = navigator.clone();
        let id = (&level).id.clone();
        Callback::from(move |_: MouseEvent| {
            navigator.push(&Route::Game { level: id.clone() });
        })
    };
    let author_onclick = {
        let navigator = navigator.clone();
        let id = (&level).item.author.clone();
        Callback::from(move |_: MouseEvent| {
            navigator.push(&Route::User { id: id.clone() });
        })
    };
    let edit_onclick = {
        let navigator = navigator.clone();
        let id = (&level).id.clone();
        Callback::from(move |_: MouseEvent| {
            navigator.push(&Route::EditLevel { level: id.clone() });
        })
    };

    let author_name = {
        let pb = pb.clone();
        let author_id = level.item.author.clone();
        use_future(|| async move {
            let author = pb.view::<User>("users", &author_id).await;
            if let Ok(author) = author {
                Some(author.item.name.clone())
            } else {
                Some("Failed to get username".to_string())
            }
        })
    };

    html! {
        <div class="puzzleCard">
            <img src="https://placehold.co/320x240" />
            <div class="puzzleInfo">
                <h1>{&level.item.name}</h1>
                if let Ok(author_name) = author_name {
                    if let Some(author_name) = &*author_name {
                        <h3 onclick={author_onclick}>{author_name}</h3>
                    }
                }
            </div>

            if let Some(auth_store) = &pb.auth_store {
                if auth_store.record.id == level.item.author {
                    <button onclick={edit_onclick}> { "Editar" } </button>
                }
            }
            <button {onclick}>{"Jogar"}</button>
        </div>
    }
}

#[autoprops]
#[component]
pub fn LevelList(pb: Rc<Pocketbase>, levels: &Vec<Record<GameLevel>>) -> Html {
    html! {
        <div class="puzzleList">
        for level in levels {
            <Level level={level.clone()} pb={pb.clone()}/>
        }

        </div>
    }
}

#[autoprops]
#[component]
pub fn LevelFetcher(pb: Rc<Pocketbase>, page: usize) -> HtmlResult {
    let levels = {
        let pb = pb.clone();
        use_future(|| async move { pb.list::<GameLevel>("levels", page, 25, None).await })?
    };
    match &*levels {
        Ok(levels) => Ok(html! {
            <>
                <div style="margin: auto 5%">
                    <h1 class="puzzlesHeader">{ "Puzzles" }</h1>
                    <LevelList pb={pb.clone()} levels={levels.items.clone()} />
                </div>
            </>
        }),
        Err(_) => Ok(html! {
            <p>{format!("Error fetching levels")}</p>
        }),
    }
}

#[autoprops]
#[component]
pub fn Home(pb: Rc<Pocketbase>) -> Html {
    html! {
        <Suspense fallback={html! {<p>{"Loading Levels..."}</p>}}>
        <LevelFetcher pb={pb.clone()} page={0} />
        </Suspense>
    }
}

#[autoprops]
#[component]
pub fn UserLevelFetcher(pb: Rc<Pocketbase>, id: AttrValue, page: usize) -> HtmlResult {
    let levels = {
        let pb = pb.clone();
        let filter = ListParams::new().filter(&format!(r#"author="{}""#, id));
        use_future(|| async move {
            pb.list::<GameLevel>("levels", page, 25, Some(&filter))
                .await
        })?
    };
    match &*levels {
        Ok(levels) => Ok(html! {
            <>
                <div style="margin: auto 5%">
                    <h1 class="puzzlesHeader">{ "Puzzles" }</h1>
                    <LevelList pb={pb.clone()} levels={levels.items.clone()} />
                </div>
            </>
        }),
        Err(_) => Ok(html! {
            <p>{format!("Error fetching levels")}</p>
        }),
    }
}

#[autoprops]
#[component]
pub fn UserPage(pb: Rc<Pocketbase>, id: AttrValue) -> Html {
    html! {
        <Suspense fallback={html! {<p>{"Loading Levels..."}</p>}}>
        <UserLevelFetcher id={id} pb={pb.clone()} page={0} />
        </Suspense>
    }
}
