use syntax::ast::{HasBindings};
use syntax::{SyntaxNode, SyntaxNodePtr};
use syntax::ast::AstNode;
use dioxus::prelude::*;
use mockall_double::double;

#[double]
use crate::components::expression::components as expression_components;

#[double]
use crate::ast::functions as ast_functions;

#[component]
pub fn AttributeSetUI(ptr: ReadSignal<SyntaxNodePtr>, nesting_level: u16) -> Element {
    let set = ast_functions::use_ast_node::<syntax::ast::AttrSet>(ptr);
    let elements = set.read().bindings()
        .filter_map(|binding| match binding {
            syntax::ast::Binding::AttrpathValue(attr) => Some(attr),
            _ => None,
                    })
        .map(|attr| {
            let label = attr.attrpath()
                .map(|ap| ap.syntax().text().to_string())
                .unwrap_or("unknown".to_string());
            let value = attr.value().unwrap();
            let node = value.syntax();
            let ptr = SyntaxNodePtr::new(node);
        rsx! {
            div {
                class: "attribute-item",
                label {
                    class: "attribute-label",
                    "{label}"
                }
                expression_components::ExpressionUI { ptr: ptr, nesting_level: nesting_level }
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

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use super::*;
    use crate::ast::mock_functions::use_ast_node_context;
    use crate::components::expression::mock_components::ExpressionUI_context;

    #[test]
    fn test_attribute_set_ui() {
        let use_ast_node_ctx = use_ast_node_context();
        let expression_ui_ctx = ExpressionUI_context();
        const SOURCE: &str = r#"
        {
            a = 1;
            b = 2;
        }
        "#;
        use_ast_node_ctx.expect()
            .returning(|_| {
                Memo::new(|| {
                    let syntax_node = syntax::parse_file(SOURCE).syntax_node();
                    let expr = syntax::ast::SourceFile::cast(syntax_node).unwrap().expr().unwrap();
                    syntax::ast::AttrSet::cast(expr.syntax().clone()).unwrap()
                })
            });
        expression_ui_ctx.expect()
            .returning(|props| {
            rsx! { div { "ExpressionUI for props: {props:?}" } }
            });
        let mut vdom = VirtualDom::new(|| {
            let syntax_node = syntax::parse_file(SOURCE).syntax_node();
            let ptr_signal = Signal::new(syntax::SyntaxNodePtr::new(&syntax_node));
             rsx! { AttributeSetUI { ptr: ptr_signal, nesting_level: 1 } }
        });
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert_snapshot!(html);
    }
}
