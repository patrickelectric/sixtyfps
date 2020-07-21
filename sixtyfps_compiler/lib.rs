/*!
# The SixtyFPS compiler library

**NOTE:** This library is an internal crate for the SixtyFPS project.
This crate should not be used directly by application using SixtyFPS.
You should use the `sixtyfps` crate instead

*/

// It would be nice to keep the compiler free of unsafe code
#![deny(unsafe_code)]

#[cfg(feature = "proc_macro_span")]
extern crate proc_macro;

use std::{cell::RefCell, rc::Rc};

pub mod diagnostics;
pub mod expression_tree;
pub mod generator;
pub mod layout;
pub mod object_tree;
pub mod parser;
pub mod typeregister;

mod passes {
    // Trait for the purpose of applying modifications to Expressions that are stored in various
    // data structures.
    pub trait ExpressionFieldsVisitor {
        fn visit_expressions(
            &mut self,
            visitor: impl FnMut(&mut super::expression_tree::Expression),
        );
    }

    pub mod collect_resources;
    pub mod compile_paths;
    pub mod inlining;
    pub mod lower_layout;
    pub mod lower_states;
    pub mod move_declarations;
    pub mod repeater_component;
    pub mod resolving;
    pub mod unique_id;
}

#[derive(Default)]
/// CompilationConfiguration allows configuring different aspects of the compiler.
pub struct CompilerConfiguration<'a> {
    /// Indicate whether to embed resources such as images in the generated output or whether
    /// to retain references to the resources on the file system.
    pub embed_resources: bool,
    /// The compiler will look in these paths for components used in the file to compile.
    pub include_paths: &'a [std::path::PathBuf],
}

pub fn compile_syntax_node(
    doc_node: parser::SyntaxNode,
    mut diagnostics: diagnostics::FileDiagnostics,
    compiler_config: &CompilerConfiguration,
) -> (object_tree::Document, diagnostics::BuildDiagnostics) {
    let mut build_diagnostics = diagnostics::BuildDiagnostics::default();

    let global_type_registry = typeregister::TypeRegister::builtin();
    let type_registry =
        Rc::new(RefCell::new(typeregister::TypeRegister::new(&global_type_registry)));

    for (path, source) in library::widget_library() {
        build_diagnostics.add(typeregister::TypeRegister::add_type_from_source(
            &type_registry,
            source.to_string(),
            &path,
        ));
    }

    if !compiler_config.include_paths.is_empty() {
        build_diagnostics.extend(
            compiler_config
                .include_paths
                .iter()
                .map(|path| {
                    let path = if path.is_relative() {
                        let mut abs_path = diagnostics.current_path.as_ref().clone();
                        abs_path.pop();
                        abs_path.push(path);
                        abs_path
                    } else {
                        path.clone()
                    };
                    typeregister::TypeRegister::add_from_directory(&type_registry, path)
                })
                .filter_map(Result::ok)
                .flatten(),
        );
    };

    let doc = crate::object_tree::Document::from_node(doc_node, &mut diagnostics, &type_registry);

    build_diagnostics.add(diagnostics);

    run_passes(&doc, &mut build_diagnostics, compiler_config);

    (doc, build_diagnostics)
}

pub fn run_passes(
    doc: &object_tree::Document,
    diag: &mut diagnostics::BuildDiagnostics,
    compiler_config: &CompilerConfiguration,
) {
    passes::resolving::resolve_expressions(doc, diag);
    passes::inlining::inline(doc);
    passes::compile_paths::compile_paths(&doc.root_component, &doc.local_registry, diag);
    passes::unique_id::assign_unique_id(&doc.root_component);
    passes::lower_layout::lower_layouts(&doc.root_component, diag);
    if compiler_config.embed_resources {
        passes::collect_resources::collect_resources(&doc.root_component);
    }
    passes::lower_states::lower_states(&doc.root_component, diag);
    passes::repeater_component::create_repeater_components(&doc.root_component);
    passes::move_declarations::move_declarations(&doc.root_component);
}

mod library {
    include!(env!("SIXTYFPS_WIDGETS_LIBRARY"));
}
