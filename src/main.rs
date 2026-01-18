use dioxus::prelude::*;
use syntax::{match_ast};
use syntax::ast::AstNode;
use std::fs;

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
    let mut file_path = use_signal(|| {"./example.nix".to_owned()});
    let contents = fs::read_to_string(file_path.read().clone()).expect("Could not read file");
    let mut ast = use_signal(|| {syntax::parse_file(&contents).syntax_node()});
    let analysis_host = hooks::use_derivation(move || {
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
        ide::AnalysisHost::new_single_file(&serialized)
    });
    use_context_provider(|| ast);
    use_context_provider(|| analysis_host);
    let root = ast.read();
    println!("AST: {}", root);
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS } 
        div {
            class: "app-container",
            h1 { "Declaro" }
            input {
                type: "file",
                id: "open-file",
                multiple: "false",
                id: "open-file",
                accept: ".nix",
                onchange: move |e| {
                    if let Some(file) = e.files() {
                        let files = file.files();
                        if let Some(path) = files.iter().next() {
                            file_path.set(path.clone());
                            let new_contents = fs::read_to_string(&path.clone()).expect("Could not read file");
                            ast.set(syntax::parse_file(&new_contents).syntax_node());
                        }
                    }
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
            Router::<router::Route> {}
        }
    }
}
