use dioxus::prelude::*;
use syntax::ast::HasBindings;
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
    let file_path = use_hook(|| {"./flake.nix"});
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
        AttributeSetUI { set: set }
    }
}

#[component]
pub fn AttributeSetUI(set: syntax::ast::AttrSet) -> Element {
    let elements = set.bindings()
        .filter_map(|binding| match binding {
            syntax::ast::Binding::AttrpathValue(attr) => attr.attrpath(),
            _ => None,
                    })
    .map(|attrpath| attrpath.syntax().text().to_string())
        .map(|attr| {
        rsx! {
            span {{ attr }}
        }
    });
    rsx! {
        div {
            id: "hero",
            { elements }
        }
    }
}




