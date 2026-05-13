use pm::pre::*;

#[proc_macro_attribute]
pub fn main(attr: TokenStream, input: TokenStream) -> TokenStream {
    pm::flatten(expand_main(attr, input))
}

fn expand_main(_attr: TokenStream, input: TokenStream) -> pm::Result<TokenStream2> {
    let item: syn::ItemFn = syn::parse(input)?;
    let old_main_name = &item.sig.ident;

    let expanded = pm::quote! {
        #[unsafe(no_mangle)]
        extern "C" fn __megaton_rs_main() {
            // Call init code
            megaton::librs_init();

            // Call user defined main
            #old_main_name();
        }
        #item
    };

    Ok(expanded)
}
