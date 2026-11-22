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

        // booleans are just identifiers with a predefined value in Nix
        // See https://github.com/NixOS/rnix-parser/issues/81
        if selected.text() == "true" || selected.text() == "false" {
            return components::BoolInput(RefInputProps { ident });
        }

        let bindings = ast_functions::get_bindings_in_scope(ident.read().syntax());
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

    #[allow(non_snake_case)]
    pub fn BoolInput(props: RefInputProps) -> Element {
        let ident = props.ident;
        let selected = ident.read().ident_token().expect("Expected an Ident token").text() == "true";
        rsx! {
            input {
            class: "bool-input simple-input",
            type: "checkbox",
            checked: "{selected}",
            onchange: move |e| {
                let new_value = if e.value() == "on" { "true" } else { "false" };
                ast_functions::update_node_value(
                    ident.read().syntax().clone().into(),
                    new_value
                );
            }
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
            .returning(|_| vec!["foo".to_string(), "bar".to_string()]);

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
    fn test_bool_input() {
        let mut vdom = VirtualDom::new(|| {
            let root = rnix::Root::parse("true");
            let ident = Ident::cast(root.tree().expr().unwrap().syntax().clone()).unwrap();
             rsx! { components::RefInput { ident } }
        });
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert_snapshot!(html);
    }
}
