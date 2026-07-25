use syntax::ast::{HasBindings};
use syntax::SyntaxNodePtr;
use syntax::ast::AstNode;
use dioxus::prelude::*;
use mockall_double::double;
use focusable_macro::focusable;
use crate::ast::functions::update_node_value;
use std::iter::zip;

#[double]
use crate::components::expression::components as expression_components;

#[double]
use crate::ast::hooks as ast_hooks;

#[component]
pub fn AttributeSetUI(ptr: ReadSignal<SyntaxNodePtr>, nesting_level: u16) -> Element {
    let set = ast_hooks::use_ast_node::<syntax::ast::AttrSet>(ptr);
    let bindings = set.read().bindings();
    let bindings_clone = bindings.clone();
    let enumerated = bindings.clone().enumerate();
    let focus = use_signal::<Option<i8>>(|| None);
    let labels = focusable!({
        iterator = enumerated,
        focus = focus,
        arms = [
        {
            matcher = syntax::ast::Binding::AttrpathValue(attr),
            focused = {
                element_type = input,
                preparation = {
                    let label = attr.attrpath()
                        .map(|ap| ap.syntax().text().to_string())
                        .unwrap_or("unknown".to_string());
                },
                content = {
                    class: "attribute-label",
                    value: "{label.trim()}",
                    oninput: move |evt| {
                        let new_label = evt.value().clone();
                        let value = attr.value().unwrap();
                        let new_attribute_set = format!("{{{}}}",enumerated.clone().map(|(i, binding)| {
                            if i == indexed_part.0 {
                                format!("{} = {};", new_label, value.syntax().text())
                            } else {
                                match binding {
                                    syntax::ast::Binding::AttrpathValue(attr) => {
                                        let label = attr.attrpath()
                                            .map(|ap| ap.syntax().text().to_string())
                                            .unwrap_or("unknown".to_string());
                                        let value = attr.value().unwrap();
                                        format!("{} = {};", label, value.syntax().text())
                                    },
                                    _ => "".to_string(),
                                }
                            }

                        }).collect::<Vec<_>>().join("\n"));
                        update_node_value(
                            set.read().syntax().clone(),
                            &new_attribute_set,
                            |syntax| {
                                <syntax::ast::SourceFile as AstNode>::cast(syntax.clone())
                                    .and_then(|sf| sf.expr())
                                    .map(|expr| expr.syntax().clone())
                            }
                        );
                    },
                    onfocusout: move |_| {
                        focus.set(None);
                    }
                }
            },
            blurred = {
                element_type = label,
                preparation = {
                    let label = attr.attrpath()
                        .map(|ap| ap.syntax().text().to_string())
                        .unwrap_or("unknown".to_string());
                    },
                    content = {
                        class: "attribute-label",
                        "{label}"
                    }
            }
        }
        ]
    });
    let elements = zip(labels,bindings).map(|(label, binding)| {
        let attr = match binding {
            syntax::ast::Binding::AttrpathValue(attr) => attr,
            _ => return rsx! { div { "Unsupported binding type" } },
        };
        let value = attr.value().unwrap();
        let node = value.syntax();
        let ptr = SyntaxNodePtr::new(node);
        rsx! {
            div {
                class: "attribute-item",
                {label}
                expression_components::ExpressionUI { ptr: ptr, nesting_level: nesting_level }
            }
        }
    });
    rsx! {
        div {
            class: "attribute-set",
            { elements }
            span { 
                class: "attribute-set-add",
                onclick: move |_| {
                    let new_attribute_set = bindings_clone.clone().map(|binding| {
                        match binding {
                            syntax::ast::Binding::AttrpathValue(attr) => {
                    let label = attr.attrpath()
                        .map(|ap| ap.syntax().text().to_string())
                        .unwrap_or("unknown".to_string());
                    let value = attr.value().unwrap();
                    format!("{} = {};", label, value.syntax().text())
                            },
                            _ => "".to_string(),
                        }
                    }).collect::<Vec<_>>().join("\n");
                    let new_attribute_set_with_new_binding = format!("{{{}\nnew_attr = 0;}}", new_attribute_set);
                    update_node_value(
                        set.read().syntax().clone(),
                        &new_attribute_set_with_new_binding,
                        |syntax| {
                            <syntax::ast::SourceFile as AstNode>::cast(syntax.clone())
                    .and_then(|sf| sf.expr())
                    .map(|expr| expr.syntax().clone())
                        }
                    );
                },
                "+"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use super::*;
    use crate::ast::mock_hooks::use_ast_node_context;
    use crate::components::expression::mock_components::ExpressionUI_context;
    use serial_test::serial;

    #[test]
    #[serial]
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
        use_ast_node_ctx.checkpoint();
        expression_ui_ctx.checkpoint();
        assert_snapshot!(html);
    }
}
