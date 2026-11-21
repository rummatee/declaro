use rnix::ast::{Path, PathAbs, PathRel, PathHome};
use rowan::ast::AstNode;
use dioxus::prelude::*;
use mockall_double::double;
use crate::ast::functions::update_node_value;
use rfd::AsyncFileDialog;
use std::path::{Path as StdPath, PathBuf};
use homedir::my_home;

#[double]
use crate::components::expression::components as expression_components;

#[double]
use crate::ast::hooks as ast_hooks;

#[component]
pub fn PathInput(path: ReadSignal<Path>) -> Element {
    let value = path.read().syntax().text().to_string();
    let path_signal = ast_hooks::use_path();
    let open_path = path_signal.read().clone();
    rsx! {
        div {
            class: "path-input simple-input",
            onclick: move |_| {
            let value = open_path.clone();
            async move {
                let file = AsyncFileDialog::new()
                    .pick_file()
                    .await;
                if let Some(file) = file {
                    let file = match *path.read() {
                        Path::PathAbs(_) => file.path().to_path_buf(),
                        Path::PathRel(_) => {
                            let parent = value.parent().unwrap_or_else(|| StdPath::new("/"));
                            let base_dir = std::path::absolute(parent).unwrap_or(parent.to_path_buf());
                            let path = file.path();
                            let relative_path = PathBuf::from("./").join(path.strip_prefix(base_dir.as_path()).unwrap_or(base_dir.as_path()));
                            relative_path.to_path_buf()
                        },
                        Path::PathHome(_) => {
                            let home_dir = my_home().unwrap_or(Some(StdPath::new("/").to_path_buf())).unwrap();
                            let path = file.path();
                            let relative_path = PathBuf::from("./").join(path.strip_prefix(home_dir.as_path()).unwrap_or(home_dir.as_path()));
                            relative_path.to_path_buf()
                        },
                        _ => file.path().to_path_buf(),
                    };

                    update_node_value(path.read().syntax().clone().into(), file.to_str().unwrap());
                }
            }
            },
            {value}
        }
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use super::*;
    use crate::components::expression::mock_components::ExpressionUI_context;
    use crate::ast::mock_hooks::use_path_context;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_path_input() {
        let expression_ui_ctx = ExpressionUI_context();
        const SOURCE: &str = "./example.nix";
        expression_ui_ctx.expect()
            .returning(|props| {
                rsx! { div { "ExpressionUI for props: {props:?}" } }
            });
        let use_path_ctx = use_path_context();
        use_path_ctx.expect()
            .returning(|| {
            let path = PathBuf::from("./example.txt");
            let signal = Signal::new(path);
            signal
            });
        let mut vdom = VirtualDom::new(|| {
            let root = rnix::Root::parse(SOURCE);
            let path = Path::cast(root.tree().expr().unwrap().syntax().clone()).unwrap();
            rsx! { PathInput { path } }
        });
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        expression_ui_ctx.checkpoint();
        use_path_ctx.checkpoint();
        assert_snapshot!(html);
    }
}
