use rnix::ast::Ident;
use rowan::ast::AstNode;
use dioxus::prelude::*;
use mockall_double::double;

#[double]
use crate::ast::functions as ast_functions;

#[cfg(test)]
use mockall::automock;

#[derive(Props, PartialEq, Clone)]
pub struct RefInputProps {
    ident: ReadSignal<Ident>,
}

#[cfg_attr(test, automock)]
pub mod components { 

    use super::*;

    #[allow(non_snake_case)]
    pub fn RefInput(props: RefInputProps) -> Element {
        let ident = props.ident;
        let selected = ident.read().ident_token().expect("Expected an Ident token");

        let bindings = ast_functions::get_bindings_in_scope(ident.read().syntax()).unwrap_or_default();
        let options = bindings
            .iter()
            .map(|label| {
                rsx! {
                    option {
                        selected: label == selected.text(),
                        { label.clone() }
                    }
                }
            });

        rsx! {
            select { 
                class: "ref-input simple-input",
                onchange: move |e| {

                    ast_functions::update_node_value(
                        ident.read().syntax().clone().into(),
                        &e.value()
                    );
                },
                {options}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use serial_test::serial;
    use super::*;

    #[test]
    #[serial]
    fn test_ref_input() {
        let get_bindings_in_scope_ctx = crate::ast::mock_functions::get_bindings_in_scope_context();
        get_bindings_in_scope_ctx.expect()
            .returning(|_| Ok(vec!["foo".to_string(), "bar".to_string()]));

        let mut vdom = VirtualDom::new(|| {
            let root = rnix::Root::parse("foo");
            let ident = Ident::cast(root.tree().expr().unwrap().syntax().clone()).unwrap();

             rsx! { components::RefInput { ident } }
        });
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert_snapshot!(html);
        get_bindings_in_scope_ctx.checkpoint();

    }

    #[test]
    #[serial]
    fn test_ref_input_get_binding_error() {
        let get_bindings_in_scope_ctx = crate::ast::mock_functions::get_bindings_in_scope_context();
        get_bindings_in_scope_ctx.expect()
            .returning(|_| Err(crate::ast::BindingsRetrievalError::ExprId));
        let mut vdom = VirtualDom::new(|| {
            let root = rnix::Root::parse("foo");
            let ident = Ident::cast(root.tree().expr().unwrap().syntax().clone()).unwrap();
             rsx! { components::RefInput { ident } }
        });
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert_snapshot!(html);
        get_bindings_in_scope_ctx.checkpoint();
    }
}
