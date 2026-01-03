use dioxus::prelude::*;
use syntax::ast::{HasBindings, HasStringParts};
use syntax::match_ast;
use syntax::ast::AstNode;
use std::fs;



const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const HEADER_SVG: Asset = asset!("/assets/header.svg");


fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let file_path = use_hook(|| {"./example.nix"});
    let contents = fs::read_to_string(file_path).expect("Could not read file");
    let ast = syntax::parse_file(&contents);
    let root = ast.syntax_node();
    print!("{:#?}",root);
    let sourceFile = match_ast!{
        match root {
        syntax::ast::SourceFile(src) => src,
        _ => panic!("Expected an source file at the root of the file, got {:?}", root.kind()),
        }
    };
    let expr = sourceFile.expr().unwrap();
    let node = expr.syntax();
    let set = match_ast!{
        match node {
        syntax::ast::AttrSet(set) => set,
        _ => panic!("Expected an attribute set at the root of the file, got {:?}", node.kind()),
        }
    };
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS } 
        div {
            class: "app-container",
            h1 { "Nix Attribute Set Editor" }
            AttributeSetUI { set: set }
        }
    }
}

#[component]
pub fn AttributeSetUI(set: syntax::ast::AttrSet) -> Element {
    let elements = set.bindings()
        .filter_map(|binding| match binding {
            syntax::ast::Binding::AttrpathValue(attr) => Some(attr),
            _ => None,
                    })
        .map(|attr| {
            let label = attr.attrpath()
                .map(|ap| ap.syntax().text().to_string())
                .unwrap_or("unknown".to_string());
            let valueUI = match attr.value() {
                    Some(val) => {
                        let node = val.syntax();
                        match_ast!{
                        match node {
                            syntax::ast::String(str_node) => {
                                rsx! { StringInput { node: str_node, id: format!("input-{}", label) } }
                            },
                            _ => rsx! { div { "Unsupported Value Type" } },
                        }
                    }},
                    _ => return rsx! { div { "Unsupported" } }
                };
        rsx! {
            div {
                class: "attribute-item",
                label {
                    class: "attribute-label",
                    "{label}"
                }
                {valueUI}
            }
        }
    });
    rsx! {
        div {
            class: "attribute-set",
            { elements }
        }
    }
}


#[component]
pub fn StringInput(node: syntax::ast::String, id: String) -> Element {
    let value = node.string_parts().filter_map(|part| {
        match part {
        syntax::ast::StringPart::Fragment(text) => Some(text.text().to_string()),
        _ => None,
        }
    }).collect::<Vec<String>>().join("");
    rsx! {
        input {
            class: "string-input",
            id: "{id}",
            value: value,
        }
    }
}

