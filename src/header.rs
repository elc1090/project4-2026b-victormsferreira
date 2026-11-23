use crate::Route;
use crate::pb::Pocketbase;
use std::rc::Rc;
use yew::{Callback, prelude::*};
use yew_autoprops::autoprops;
use yew_router::prelude::*;
#[autoprops]
#[component]
pub fn Header(pb: Rc<Pocketbase>, on_logout: Callback<()>) -> Html {
    let authstore = if let Some(auth) = pb.auth_store.clone() {
        Some(auth.record)
    } else {
        None
    };

    html! {
        <ul class="headerBar">
            <ul class="headerBar">
                <li class="floatLeft"><Link<Route> to={Route::Home}>{ "Projeto4" }</Link<Route>></li>
            </ul>
            <ul class="headerBar">
            if let Some(record) = authstore {
                <li class="floatRight"><Link<Route> to={Route::User { id: record.id.clone() }}>{ record.name.clone() }</Link<Route>></li>
                <li class="floatRight"><a onclick={Callback::from(move |e: MouseEvent| { e.prevent_default(); on_logout.emit(())})}>{"Logout"}</a></li>
            } else  {
                <li class="floatRight"><Link<Route> to={Route::Login}>{ "Login" }</Link<Route>></li>
                <li class="floatRight"><Link<Route> to={Route::SignUp}>{ "Registro" }</Link<Route>></li>
            }
             </ul>
        </ul>
    }
}
