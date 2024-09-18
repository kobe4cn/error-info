use darling::{ast::Data, util, FromDeriveInput, FromVariant};
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(error_info))]
struct ErrorData {
    ident: syn::Ident,
    generics: syn::Generics,
    data: darling::ast::Data<EnumVariants, ()>,
    app_type: syn::Type,
    prefix: String,
}
#[allow(dead_code)]
#[derive(Debug, FromVariant)]
#[darling(attributes(error_info))]
struct EnumVariants {
    ident: syn::Ident,
    fields: darling::ast::Fields<util::Ignored>,
    code: String,
    #[darling(default)]
    app_code: String,
    #[darling(default)]
    client_msg: String,
}

pub(crate) fn process_error_info(input: syn::DeriveInput) -> TokenStream {
    let ErrorData {
        ident: name,
        generics,
        data: Data::Enum(data),
        app_type,
        prefix,
    } = ErrorData::from_derive_input(&input).expect("can not parse input")
    else {
        panic!("only enum is supported");
    };
    let variants = data
        .into_iter()
        .map(|v| {
            let EnumVariants {
                ident,
                fields: _,
                code,
                app_code,
                client_msg,
            } = v;
            let code = format!("{}{}", prefix, code);
            quote! {
                #name::#ident(v) => {
                    ErrorInfo::try_new(
                        #app_code,
                        #code,
                        #client_msg,
                        self,
                    )
                }
            }
        })
        .collect::<Vec<_>>();
    quote! {
        use error_code::{ErrorInfo, ToErrorInfo as _};
        impl #generics ToErrorInfo for #name #generics {
            type T = #app_type;
            fn to_error_info(&self) -> Result<ErrorInfo<Self::T>, <Self::T as FromStr>::Err> {
                match self {
                    #(#variants)*
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_struct() {
        let good_input = r#"
        #[derive(thiserror::Error, ToErrorInfo)]
        #[error_info(app_type="http::StatusCode", prefix="01")]
        pub enum MyError {
        #[error("Invalid command: {0}")]
        #[error_info(code="IC", app_code="400")]
        InvalidCommand(String),

        #[error("Invalid argument: {0}")]
        #[error_info(code="IA", app_code="400", client_msg="friendly msg")]
        InvalidArgument(String),

        #[error("{0}")]
        #[error_info(code="RE", app_code="500")]
        RespError(#[from] RespError),
        }
        "#;
        let parsed = syn::parse_str(good_input).unwrap();
        let info = ErrorData::from_derive_input(&parsed).unwrap();
        println!("info +++++++{:#?}", info);
    }
}
