use dioxus::prelude::*;

use crate::components::NodeUI;
use crate::ast::AstPath;

#[derive(Clone, Debug, PartialEq, Routable)]
pub enum Route {

    #[route("/node/:path")]
    NodeUI { path: AstPath },

    #[route("/")]
    Home {}

}

#[component]
pub fn Home() -> Element {
    let nav = navigator();

    use_effect(move || {
        nav.replace(Route::NodeUI {
            path: AstPath { indices: vec![0] },
        });
    });

    rsx! {
        div {
            "Redirecting to root node..."
        }
    }
}
