use dioxus::prelude::*;
use dioxus_motion::prelude::*;
use dioxus_motion::transitions::page_transitions::TransitionVariantResolver;

use crate::components::NodeUI;
use crate::ast::AstPath;

#[derive(Clone, Debug, PartialEq, Routable, MotionTransitions)]
pub enum Route {

    #[layout(Wrapper)]
        #[route("/node/:path")]
        #[transition(SlideRight)]
        NodeUI { path: AstPath },

        #[route("/")]
        Home {}

}

#[component]
pub fn Wrapper() -> Element {
    let resolver: TransitionVariantResolver<Route> = std::rc::Rc::new(|from, to| {
        match (from, to) {
            (Route::NodeUI { path: from_path }, Route::NodeUI { path: to_path }) => {
                if from_path.indices.len() > to_path.indices.len() {
                    TransitionVariant::SlideRight
                } else if from_path.indices.len() < to_path.indices.len() {
                    TransitionVariant::SlideLeft
                } else {
                    TransitionVariant::Fade
                }
            }
            _ => TransitionVariant::Fade
        }
    });
    use_context_provider(|| resolver);
    rsx! {
        AnimatedOutlet::<Route> {}
    }
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
