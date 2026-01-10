use dioxus::prelude::*;
use syntax::ast::{HasBindings, HasStringParts};
use syntax::{match_ast, SyntaxNode, SyntaxNodePtr};
use syntax::ast::AstNode;
use std::fs;



const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const HEADER_SVG: Asset = asset!("/assets/header.svg");

#[macro_export]
macro_rules! use_ast_node_strict {
    (
        $ast:expr,
        $ptr:expr => $ty:ty
    ) => {{
        use dioxus::prelude::*;
        use rowan::ast::AstNode;

        use_memo(move || {
            let ast = $ast.read();
            let syntax = $ptr.to_node(&ast);

            <$ty as AstNode>::cast(syntax)
                .expect("AST node cast failed")
        })
    }};
}

fn update_node_value<N, F>(
    mut ast: Signal<SyntaxNode>,
    node: N,
    new_value: &str,
    extract_new_node: F,
)
where
    N: AstNode,
    F: Fn(&SyntaxNode) -> Option<SyntaxNode>,
{
    let new_syntax = syntax::parse_file(new_value).syntax_node();
    if let Some(new_node) = extract_new_node(&new_syntax) {
        let green_node = new_node.green();
        let new_root = SyntaxNode::new_root(
            node.syntax().replace_with((*green_node).to_owned())
        );
        ast.set(new_root);
    }
}

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut file_path = use_signal(|| {"./example.nix".to_owned()});
    let contents = fs::read_to_string(file_path.read().clone()).expect("Could not read file");
    let mut ast = use_signal(|| {syntax::parse_file(&contents).syntax_node()});
    println!("AST: {}", ast.read());
    let root = ast.read();
    let sourceFile = match_ast!{
        match root {
        syntax::ast::SourceFile(src) => src,
        _ => panic!("Expected an source file at the root of the file, got {:?}", root.kind()),
        }
    };
    let expr = sourceFile.expr().unwrap();
    let node = expr.syntax();
    let ptr = SyntaxNodePtr::new(&node);
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS } 
        div {
            class: "app-container",
            h1 { "Declaro" }
            input {
                type: "file",
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
            AttributeSetUI { ptr: ptr, ast: ast }
        }
    }
}

#[component]
pub fn AttributeSetUI(ptr: SyntaxNodePtr, ast: Signal<SyntaxNode>) -> Element {
    let set = use_ast_node_strict!(ast, ptr => syntax::ast::AttrSet);
    let elements = set.read().bindings()
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
                            syntax::ast::String(_str_node) => {
                                let ptr = SyntaxNodePtr::new(&node);
                                rsx! { StringInput { ptr: ptr, ast: ast, id: format!("input-{}", label) } }
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
pub fn StringInput(ptr: SyntaxNodePtr, ast: Signal<SyntaxNode>, id: String) -> Element {
    let node = use_ast_node_strict!(ast, ptr => syntax::ast::String);
    let value = node.read().string_parts().filter_map(|part| {
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
            oninput: move |e| {
                println!("New value: {}", e.value());
                update_node_value(
                    ast,
                    node.read().clone(),
                    &format!("\"{}\"", e.value()),
                    |syntax| {
                        <syntax::ast::SourceFile as AstNode>::cast(syntax.clone())
                            .and_then(|sf| sf.expr())
                            .map(|expr| expr.syntax().clone())
                    }
                );
            }
        }
    }
}

