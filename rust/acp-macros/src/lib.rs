use convert_case::Casing;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    Error, Expr, Field, FnArg, Ident, ItemTrait, Pat, ReturnType, TraitItem, Type,
    parse_macro_input,
};

#[proc_macro_attribute]
pub fn acp_peer(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut peer_trait = parse_macro_input!(item as ItemTrait);

    let mut methods = Vec::with_capacity(peer_trait.items.len());
    let mut io_types = Vec::with_capacity(peer_trait.items.len() * 2);
    let mut any_req_members = Vec::with_capacity(peer_trait.items.len());
    let mut any_res_members = Vec::with_capacity(peer_trait.items.len());
    let mut call_branches = Vec::with_capacity(peer_trait.items.len());

    let any_req_name = Ident::new(
        &format!("Any{}Request", peer_trait.ident),
        Span::call_site(),
    );
    let any_res_name = Ident::new(
        &format!("Any{}Response", peer_trait.ident),
        Span::call_site(),
    );

    for item in peer_trait.items.iter_mut() {
        let TraitItem::Fn(fn_def) = item else {
            continue;
        };

        let method_name = &fn_def.sig.ident;
        let method_name_str = method_name.to_string();
        methods.push(method_name_str.clone());

        let mut param_fields: Vec<Field> = Vec::with_capacity(fn_def.sig.inputs.len());
        let mut param_args: Vec<Expr> = Vec::with_capacity(fn_def.sig.inputs.len());

        for arg in fn_def.sig.inputs.iter() {
            match arg {
                FnArg::Receiver(_receiver) => {}
                FnArg::Typed(pat_type) => {
                    let Pat::Ident(arg_name) = &*pat_type.pat else {
                        return TokenStream::from(
                            Error::new_spanned(
                                pat_type,
                                "Only simple identifiers are supported as function arguments",
                            )
                            .to_compile_error(),
                        );
                    };
                    let ty = &pat_type.ty;
                    let field_ident = &arg_name.ident;
                    let field_type = &ty;
                    param_fields.push(syn::parse_quote! {
                        pub #field_ident: #field_type
                    });
                    param_args.push(syn::parse_quote! {
                        params.#field_ident
                    });
                }
            }
        }

        let method_pascal = method_name_str.to_case(convert_case::Case::Pascal);

        let response_type_name =
            Ident::new(&format!("{}Response", &method_pascal), Span::call_site());

        any_res_members.push(quote! {
            #response_type_name(#response_type_name)
        });

        match &fn_def.sig.output {
            ReturnType::Default => io_types.push(quote! {
                #[derive(Debug, ::serde::Serialize, ::serde::Deserialize, ::schemars::JsonSchema)]
                struct #response_type_name
            }),
            ReturnType::Type(_, ty) => match &**ty {
                Type::Tuple(tup) if tup.elems.is_empty() => io_types.push(quote! {
                    #[derive(Debug, ::serde::Serialize, ::serde::Deserialize, ::schemars::JsonSchema)]
                    struct #response_type_name
                }),
                Type::Path(path) => {
                    if path.path.segments.last().unwrap().ident != response_type_name {
                        return TokenStream::from(
                            Error::new_spanned(
                                path,
                                format!("return type must be named `{}`", response_type_name),
                            )
                            .to_compile_error(),
                        );
                    }
                },
                _ => {
                    return TokenStream::from(
                        Error::new_spanned(
                            &fn_def.sig.output,
                            format!("Must return a type named `{}` or `()`,", response_type_name),
                        )
                        .to_compile_error(),
                    );
                }
            }
        }

        let params_type_name = Ident::new(&format!("{}Params", method_pascal), Span::call_site());
        let params = quote! {
            #[derive(Debug, ::serde::Serialize, ::serde::Deserialize, ::schemars::JsonSchema)]
            #[serde(rename_all = "camelCase")]
            struct #params_type_name {
                #(#param_fields),*
            }
        };
        io_types.push(params);
        any_req_members.push(quote! {
            #params_type_name(#params_type_name)
        });

        call_branches.push(quote! {
            #any_req_name::#params_type_name(params) => {
                let response = self.#method_name(#(#param_args),*).await;
                Ok(#any_res_name::#response_type_name(response))
            }
        });
    }

    let methods_const_name = Ident::new(
        &format!("{}_METHODS", peer_trait.ident.to_string().to_uppercase()),
        Span::call_site(),
    );

    let any_req_enum = quote! {
        #[derive(::serde::Deserialize, ::serde::Serialize, ::schemars::JsonSchema)]
        #[serde(untagged)]
        enum #any_req_name {
           #(#any_req_members),*
        }
    };

    let any_res_enum = quote! {
        #[derive(::serde::Deserialize, ::serde::Serialize, ::schemars::JsonSchema)]
        #[serde(untagged)]
        enum #any_res_name {
           #(#any_res_members),*
        }
    };

    peer_trait.items.push(syn::parse_quote! {
        async fn call_any(&self, params: #any_req_name) -> Result<#any_res_name> {
            match params {
                #(#call_branches)*
            }
        }
    });

    TokenStream::from(quote! {
        #[::async_trait::async_trait(?Send)]
        #peer_trait

        #any_req_enum

        #any_res_enum

        const #methods_const_name: &[&str] = &[#(#methods),*];

        #(#io_types)*
    })
}
