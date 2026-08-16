//! Procedural macros for compiling KDL UI layouts into `no_std` Rust code for `embedded-gui`.

use proc_macro::TokenStream;
use std::path::PathBuf;
use syn::{LitStr, parse_macro_input};

/// Compiles an external KDL UI markup file into typed, zero-allocation Rust code.
///
/// # Example
/// ```ignore
/// embedded_gui_macros::include_gui!("ui/main_screen.kdl");
/// ```
#[proc_macro]
pub fn include_gui(input: TokenStream) -> TokenStream {
    let lit_str = parse_macro_input!(input as LitStr);
    let rel_path = lit_str.value();

    let manifest_dir = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(dir) => PathBuf::from(dir),
        Err(_) => {
            return syn::Error::new(lit_str.span(), "Failed to determine CARGO_MANIFEST_DIR")
                .to_compile_error()
                .into();
        }
    };

    let full_path = manifest_dir.join(&rel_path);
    let kdl_source = match std::fs::read_to_string(&full_path) {
        Ok(content) => content,
        Err(err) => {
            return syn::Error::new(
                lit_str.span(),
                format!(
                    "Failed to read KDL file at '{}': {}",
                    full_path.display(),
                    err
                ),
            )
            .to_compile_error()
            .into();
        }
    };

    match embedded_gui_codegen::compile_kdl_to_rust(&kdl_source) {
        Ok(rust_code) => match rust_code.parse::<proc_macro2::TokenStream>() {
            Ok(tokens) => tokens.into(),
            Err(err) => syn::Error::new(
                lit_str.span(),
                format!("Failed to tokenize generated code: {}", err),
            )
            .to_compile_error()
            .into(),
        },
        Err(err) => syn::Error::new(lit_str.span(), format!("KDL GUI Codegen Error: {}", err))
            .to_compile_error()
            .into(),
    }
}

/// Compiles an inline KDL string literal into typed Rust code.
///
/// # Example
/// ```ignore
/// embedded_gui_macros::gui_kdl!(r#"
/// screen id="Thermostat" width=320 height=240 {
///     grid cols="140px 1fr" rows="24px 1fr 48px" gap=6 {
///         banner col=0 row=0 col_span=2 text="Smart Thermostat"
///         spinbox id="Temp" col=0 row=1 min=100 max=350 value=215
///         scale id="Gauge" col=1 row=1 mode="radial" min=10.0 max=40.0 value=22.5
///     }
/// }
/// "#);
/// ```
#[proc_macro]
pub fn gui_kdl(input: TokenStream) -> TokenStream {
    let lit_str = parse_macro_input!(input as LitStr);
    let kdl_source = lit_str.value();

    match embedded_gui_codegen::compile_kdl_to_rust(&kdl_source) {
        Ok(rust_code) => match rust_code.parse::<proc_macro2::TokenStream>() {
            Ok(tokens) => tokens.into(),
            Err(err) => syn::Error::new(
                lit_str.span(),
                format!("Failed to tokenize generated code: {}", err),
            )
            .to_compile_error()
            .into(),
        },
        Err(err) => syn::Error::new(lit_str.span(), format!("KDL GUI Codegen Error: {}", err))
            .to_compile_error()
            .into(),
    }
}
