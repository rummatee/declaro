use dioxus::prelude::*;
use syntax::{match_ast};
use syntax::ast::AstNode;
use std::fs;
use std::path::PathBuf;
use rfd::AsyncFileDialog;

mod ast;
mod components;
mod router;
mod hooks;



const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut file_path = use_signal(|| {PathBuf::from("./example.nix")});
    let mut ast = hooks::use_derivation(move || {
        let contents = fs::read_to_string(file_path.read().clone()).expect("Could not read file");
        syntax::parse_file(&contents).syntax_node()
    });
    let analysis_host = hooks::use_derivation(move || {
        let root = ast.read();
        println!("AST: {}", root);
        let sourceFile = match_ast!{
            match root {
                syntax::ast::SourceFile(src) => src,
                _ => panic!("Expected an source file at the root of the file, got {:?}", root.kind()),
            }
        };
        let expr = sourceFile.expr().unwrap();
        let node = expr.syntax();
        let serialized = node.to_string();
        ide::AnalysisHost::new_single_file(&serialized)
    });
    use_context_provider(|| ast);
    use_context_provider(|| analysis_host);
    let root = ast.read();
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS } 
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
                    let root = ast.read();
                    let sourceFile = match_ast!{
                match root {
                syntax::ast::SourceFile(src) => src,
                _ => panic!("Expected an source file at the root of the file, got {:?}", root.kind()),
                }
                    };
                    let expr = sourceFile.expr().unwrap();
                    let node = expr.syntax();
                    let serialized = node.to_string();
                    fs::write(file_path.read().clone(), serialized).expect("Could not write to file");
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
