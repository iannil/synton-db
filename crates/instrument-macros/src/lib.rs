//! Procedural macros for SYNTON-DB instrumentation system.
//!
//! Provides:
//! - `#[trace]` - Function-level instrumentation macro
//! - `#[checkpoint]` - Checkpoint instrumentation macro

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, FnArg, ItemFn};

/// Function-level instrumentation macro.
///
/// # Examples
///
/// Basic usage:
/// ```rust,ignore
/// #[trace]
/// async fn my_function(x: i32, y: String) -> Result<i32> {
///     Ok(x + 1)
/// }
/// ```
#[proc_macro_attribute]
pub fn trace(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);

    // Extract function details
    let fn_vis = input.vis.clone();
    let fn_sig = input.sig.clone();
    let fn_block = input.block.clone();
    let fn_attrs = input.attrs.clone();

    let fn_name = &fn_sig.ident;
    let fn_name_str = fn_name.to_string();

    // Extract parameters for recording
    let param_names: Vec<&proc_macro2::Ident> = fn_sig
        .inputs
        .iter()
        .filter_map(|arg| match arg {
            FnArg::Typed(pat_type) => {
                if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                    Some(&pat_ident.ident)
                } else {
                    None
                }
            }
            FnArg::Receiver(_) => None,
        })
        .collect();

    // Build parameter recording for metadata
    let metadata_fields: Vec<TokenStream2> = param_names
        .iter()
        .map(|name| {
            let name_str = name.to_string();
            quote! {
                args.insert(#name_str.to_string(), std::format!("{:?}", &#name).into());
            }
        })
        .collect();

    // Check if function is async
    let is_async = fn_sig.asyncness.is_some();

    // Build the instrumented function
    let expanded = if is_async {
        quote! {
            #(#fn_attrs)*
            #fn_vis #fn_sig {
                // Create span metadata
                let mut args_metadata = std::collections::HashMap::new();
                #(#metadata_fields)*

                let span_id = synton_instrument::TraceCollector::current_span_id();
                let parent_id = synton_instrument::TraceCollector::parent_span_id();

                let _guard = synton_instrument::TraceCollector::global().enter_span(
                    #fn_name_str.to_string(),
                    parent_id,
                    synton_instrument::SpanMetadata::new(
                        #fn_name_str.to_string(),
                        module_path!().to_string(),
                        file!().to_string(),
                        line!() as u32,
                        synton_instrument::SpanKind::Function,
                    ).with_args(args_metadata),
                );

                async move {
                    #fn_block
                }.await
            }
        }
    } else {
        quote! {
            #(#fn_attrs)*
            #fn_vis #fn_sig {
                // Create span metadata
                let mut args_metadata = std::collections::HashMap::new();
                #(#metadata_fields)*

                let span_id = synton_instrument::TraceCollector::current_span_id();
                let parent_id = synton_instrument::TraceCollector::parent_span_id();

                let _guard = synton_instrument::TraceCollector::global().enter_span(
                    #fn_name_str.to_string(),
                    parent_id,
                    synton_instrument::SpanMetadata::new(
                        #fn_name_str.to_string(),
                        module_path!().to_string(),
                        file!().to_string(),
                        line!() as u32,
                        synton_instrument::SpanKind::Function,
                    ).with_args(args_metadata),
                );

                #fn_block
            }
        }
    };

    TokenStream::from(expanded)
}

/// Checkpoint instrumentation macro.
///
/// # Examples
///
/// Basic checkpoint:
/// ```rust,ignore
/// #[checkpoint]
/// fn checkpoint_name() {
///     // Checkpoint reached
/// }
/// ```
#[proc_macro_attribute]
pub fn checkpoint(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);

    // Extract function details
    let fn_vis = input.vis.clone();
    let fn_sig = input.sig.clone();
    let fn_block = input.block.clone();
    let fn_attrs = input.attrs.clone();

    let fn_name = &fn_sig.ident;
    let fn_name_str = fn_name.to_string();

    // Build the checkpoint function
    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis #fn_sig {
            let span_id = synton_instrument::TraceCollector::current_span_id();

            synton_instrument::TraceCollector::global().checkpoint(
                span_id,
                #fn_name_str.to_string(),
            );

            #fn_block
        }
    };

    TokenStream::from(expanded)
}

/// Derive macro for generating trace metadata.
#[proc_macro_derive(TraceMetadata)]
pub fn derive_trace_metadata(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    let expanded = quote! {
        impl synton_instrument::TraceMetadata for #struct_name {
            fn trace_metadata(&self) -> std::collections::HashMap<String, serde_json::Value> {
                let mut map = std::collections::HashMap::new();
                map.insert("type".to_string(), std::stringify!(#struct_name).to_string().into());
                map
            }
        }
    };

    TokenStream::from(expanded)
}
