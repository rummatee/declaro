use dioxus::prelude::*;
use rowan::ast::AstNode;
use std::fs;
use std::path::PathBuf;
use rfd::AsyncFileDialog;

mod ast;
mod components;
mod router;
mod utils;


const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut file_path = use_signal(|| {PathBuf::from("./example.nix")});
    let contents = utils::hooks::use_derivation(move || {
        fs::read_to_string(file_path.read().clone()).expect("Could not read file")
    });
    let ast = utils::hooks::use_derivation(move || {
        rnix::Root::parse(contents.read().as_str()).tree().syntax().clone()
    });
    use_context_provider(|| contents);
    use_context_provider(|| ast);
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Stylesheet { href: MAIN_CSS } 
        div {
            class: "menu",
            button {
                id: "open-file",
                id: "open-file",
                onclick: move |_| async move {
                    let file = AsyncFileDialog::new()
                        .add_filter("Nix files", &["nix"])
                        .pick_file()
                        .await.unwrap();
                    let path = file.clone().path().to_path_buf();

                    file_path.set(path.clone());
                },
                "Open"
            }
            button {
                onclick: move |_| {
                    fs::write(file_path.read().clone(), contents.read().clone()).expect("Could not write to file");
                },
                id: "save-file",
                "Save"
            }
        }
        div {
            class: "app-container",
            Router::<router::Route> {}
        }
    }
}
