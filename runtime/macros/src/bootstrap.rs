use proc_macro::TokenStream;
use syn::{ItemFn, Meta, LitStr};

type TokenStream2 = proc_macro2::TokenStream;
/// Implementation of the `#[megaton::bootstrap]` attribute.
pub fn bootstrap_impl(item: TokenStream) -> TokenStream {
    let mut parsed = syn::parse_macro_input!(item as ItemFn);

    let mut expanded = TokenStream2::new();

    // process attributes
    let mut found_module_name = false;
    let mut keep_attrs = Vec::new();

    for attr in std::mem::take(&mut parsed.attrs) {
        if let Meta::List(list) = attr.meta {
            if list.path.is_ident("module") {
                found_module_name = true;
                let module_name=TokenStream2::from(declare_module_name(list.tokens.into()));
                expanded.extend(module_name);
            } else if list.path.is_ident("abort") {
            }
            continue;
        }
        keep_attrs.push(attr);
    }

    if !found_module_name {
        panic!("Missing module name!. Please add #[module(\"...\")].");
    }

    let main_name = &parsed.sig.ident;

    // generate bootstrap
    let megaton_rust_main = quote::quote! {
        #[no_mangle]
        pub extern "C" fn megaton_rust_main() {
            // Rust side initialization
            megaton::bootstrap_rust();
            // Call main
            #main_name();
        }
    };

    expanded.extend(megaton_rust_main);
    for attr in keep_attrs {
        expanded.extend(quote::quote! { #attr });
    }

    let vis = parsed.vis;
    let sig = parsed.sig;
    let block = parsed.block;

    expanded.extend(quote::quote! {
        #vis #sig #block
    });

    expanded.into()
}

pub fn declare_module_name(attr: TokenStream) -> TokenStream {
    let literal = syn::parse_macro_input!(attr as LitStr);
    let value = literal.value();
    let len = value.len();
    let mut byte_array = TokenStream2::new();
    for byte in value.bytes() {
        byte_array.extend(quote::quote! { #byte, });
    }

    
    let out = quote::quote! {
        #[link_section = ".nx-module-name"]
        #[used]
        static NX_MODULE_NAME: megaton::ModuleName<[u8; #len]> = 
            megaton::ModuleName::new(#len as u32, [#byte_array]);
        #[no_mangle]
        pub extern "C" fn megaton_module_name() -> *const megaton::ModuleName<[u8; #len]> {
            &NX_MODULE_NAME as *const _
        }
        pub const fn module_name() -> &'static str {
            #literal
        }
    };

    out.into()
}


