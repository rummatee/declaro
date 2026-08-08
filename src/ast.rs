use ide::{AnalysisHost, FileId};
use syntax::{SyntaxNode, SyntaxNodePtr, NixLanguage, ast::AstNode, ast::AstChildren};
use dioxus::prelude::*;
use mockall_double::double;
use thiserror::Error;

#[cfg(test)]
use mockall::automock;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AstPath {
    pub indices: Vec<usize>,
}

#[cfg_attr(test, automock)]
pub mod hooks {
    use super::*;

    pub fn use_syntax_node() -> Signal<SyntaxNode> {
        use_context::<Signal<SyntaxNode>>()
    }

    pub fn use_ast_node<T>(ptr: ReadSignal<SyntaxNodePtr>) -> Memo<T>
    where
        T: AstNode<Language = NixLanguage> + PartialEq + 'static,
    {
        let ast = use_syntax_node();
        use_memo(move || {
            let syntax = ptr.read().to_node(&ast.read());
            T::cast(syntax).expect("Failed to cast syntax node to expected type")
        })
    }
}

#[cfg_attr(test, automock)]
pub mod functions {
    use super::*;

    #[double]
    use super::hooks;

    pub fn path_from_root(node: &SyntaxNode) -> AstPath {
        let mut indices = Vec::new();
        let mut current = node.clone();

        while let Some(parent) = current.parent() {
            let index = parent
                .children()
                .position(|child| child == current)
                .expect("node must be child of its parent");

            indices.push(index);
            current = parent;
        }

        indices.reverse();

        AstPath { indices }
    }

    pub fn resolve_path(root: &SyntaxNode, path: &AstPath) -> Option<SyntaxNode> {
        let mut current = root.clone();

        for &index in &path.indices {
            current = current.children().nth(index)?;
        }

        Some(current)
    }

    pub fn collect_path(root: SyntaxNode, path: &AstPath) -> Vec<IndexedNode> {
        let mut nodes = Vec::new();
        let mut current = root;
        let mut current_path = AstPath { indices: Vec::new() };
        let index_node = IndexedNode {
            index: AstPath { indices: vec![0] },
            node: current.clone(),
        };
        nodes.push(index_node);
        for &index in &path.indices {
            if let Some(child) = current.children().nth(index) {
                current_path.indices.push(index);
                let index_node = IndexedNode {
                    index: current_path.clone(),
                    node: child.clone(),
                };
                nodes.push(index_node);
                current = child;
            } else {
                break;
            }
        }
        nodes
    }

    impl std::fmt::Display for AstPath {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            for (i, idx) in self.indices.iter().enumerate() {
                if i > 0 {
                    write!(f, ".")?;
                }
                write!(f, "{idx}")?;
            }
            Ok(())
        }
    }

    impl std::fmt::Display for ParseAstPathError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "failed to parse AstPath")
        }
    }

    impl std::str::FromStr for AstPath {
        type Err = ParseAstPathError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let indices = if s.is_empty() {
                Ok(Vec::new())
            } else {
                s.split('.')
                    .map(|p| p.parse::<usize>().map_err(|_| ()))
                    .collect::<Result<Vec<_>, _>>().map_err(|_| {ParseAstPathError})
            };

            indices.map(|indices| AstPath { indices } )
        }
    }

    pub fn update_node_value(node: SyntaxNode, new_value: &str)
    {
        let range = node.text_range();
        let mut source = use_context::<Signal<String>>();
        source.write().replace_range(usize::from(range.start())..usize::from(range.end()), new_value);
    }

    pub fn add_binding<N: AstNode>(nodes: AstChildren<N>)
    {
        let mut source = use_context::<Signal<String>>();
        source.write().insert_str(nodes.last().map(|node| usize::from(node.syntax().text_range().end())).unwrap_or(0), "new_attr = 0;\n");
    }

    pub fn get_bindings_in_scope(node: &SyntaxNode, analysis: &(AnalysisHost, FileId)) -> Result<Vec<String>, BindingsRetrievalError> {
        let snapshot = analysis.0.snapshot();
        let scopes = snapshot.scopes(analysis.1)?;
        let expr_id = snapshot
            .source_map(analysis.1).unwrap()
            .expr_for_node(SyntaxNodePtr::new(node)).ok_or(BindingsRetrievalError::ExprId)?;
        let scope_id = scopes.scope_for_expr(expr_id).ok_or(BindingsRetrievalError::ScopeForExpr)?;
        Ok(scopes
            .ancestors(scope_id)
            .filter_map(|scope| scope.as_definitions())
            .flatten()
            .map(|(name, _def)| name.to_string())
            .collect::<Vec<String>>())
    }
}


#[derive(Error, Debug)]
pub enum BindingsRetrievalError {
    #[error("Failed getting scopes")]
    Scopes(#[from] ide::Cancelled),
    #[error("Failed getting expression id")]
    ExprId,
    #[error("Failed getting scope for expression")]
    ScopeForExpr,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseAstPathError;

pub struct IndexedNode {
    pub index: AstPath,
    pub node: SyntaxNode,
}

impl std::fmt::Display for IndexedNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "IndexedNode {{ index: {}, node: {:?} }}", self.index, self.node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;
    use std::str::FromStr;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_ast_path_roundtrip() {
        let code = r#"
            let x = 1;
            in {
                y = {
                    z = 2;
                };
            }
        "#;
        let ast = syntax::parse_file(code).syntax_node();
        let path = AstPath::from_str("0.1.0").expect("Failed to parse AstPath");
        let node = functions::resolve_path(&ast, &path).expect("Node should exist");
        assert_snapshot!(node);
        let collected_nodes = functions::collect_path(ast.clone(), &path);
        assert_snapshot!(collected_nodes.iter().map(|n| format!("{}", n)).collect::<String>());
        let path_from_root = functions::path_from_root(&node);
        assert_eq!(path, path_from_root);
    }

    #[test]
    #[serial]
    fn test_empty_ast_path() {
        let path = AstPath::from_str("").expect("Failed to parse empty AstPath");
        assert_eq!(path.indices.len(), 0);
    }

    #[test]
    #[serial]
    fn test_invalid_ast_path() {
        let result = AstPath::from_str("0.a.1");
        assert!(result.is_err());
        assert_eq!(result.err().unwrap().to_string(), "failed to parse AstPath");
    }

    #[test]
    #[serial]
    fn test_update_node_value() {
        // drop into dioxus environment to allow SIgnal creation
        let mut vdom = VirtualDom::new(|| {
            let code = r#"
                {
                    a = 1;
                    b = "b";
                }
            "#;
            let ast = syntax::parse_file(code).syntax_node();
            let ast_signal = Signal::new(ast.clone());
            let use_syntax_node_ctx = mock_hooks::use_syntax_node_context();
            use_syntax_node_ctx.expect()
                .return_const_st( ast_signal );
            let node = functions::resolve_path(&ast, &AstPath::from_str("0.1.1").unwrap()).expect("Node should exist");
            let new_value = "2";
            functions::update_node_value(node.clone(), new_value);
            let updated_ast = ast_signal.read().clone();
            let updated_node = functions::resolve_path(&updated_ast, &AstPath::from_str("0.1.1").unwrap()).expect("Node should exist");
            assert_eq!(updated_node.to_string(), "2");
            rsx! { "success" }
        });
        vdom.rebuild_in_place();
        assert!(dioxus_ssr::render(&vdom).contains("success"));
    }

    #[test]
    #[serial]
    fn test_get_bindings_in_scope() {
        let analysis_host = AnalysisHost::new_single_file("let foo = 1; in let bar = 2; in foo + bar").0;
        let syntax_node = syntax::parse_file("let foo = 1; in let bar = 2; in foo + bar").syntax_node();
        let path = crate::ast::AstPath{indices: vec![0, 1, 1, 0]}; // Path to the 'foo' reference in 'foo + bar'
        let expr = crate::ast::functions::resolve_path(&syntax_node, &path).unwrap();
        let ref_node = syntax::ast::Ref::cast(expr.clone()).unwrap();
        let bindings = functions::get_bindings_in_scope(ref_node.syntax(), &(analysis_host, FileId(0))).unwrap();
        assert_eq!(bindings, vec!["bar".to_string(), "foo".to_string()]);
    }
}
