//! `embedded-gui-codegen`: KDL Markup Parser and Rust Code Generator for `embedded-gui`
//!
//! Enables UI designers and non-technical domain experts to author declarative GUI screens
//! in KDL and compile them into deterministic, zero-allocation (`no_std`) Rust code.

pub mod assets;
pub mod ast;
pub mod emitter;
pub mod parser;
pub mod serializer;

#[cfg(test)]
mod tests;

pub use ast::*;
pub use emitter::*;
pub use parser::*;
pub use serializer::*;

/// Convenience function: parses KDL and returns the complete generated Rust code string.
pub fn compile_kdl_to_rust(kdl_source: &str) -> Result<String, CodegenError> {
    let screen = parse_kdl_screen(kdl_source)?;
    Ok(generate_rust_code(&screen))
}
