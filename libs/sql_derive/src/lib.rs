extern crate proc_macro;
extern crate syn;

use syn::{Body, ConstExpr, Mutability, Ty};
use proc_macro::TokenStream;
use std::str::FromStr;

#[proc_macro_derive(SqlDataObject)]
pub fn sql_data_object(input: TokenStream) -> TokenStream {
    // Construct a string representation of the type definition
    let s = input.to_string();
    
    // Parse the string representation
    let ast = syn::parse_macro_input(&s).unwrap();

    println!("Struct: {:?}", ast.ident);
    match ast.body {
        Body::Enum(ref _variants) => {
            panic!("Enums not supported yet");
        },
        Body::Struct(ref variant_data) => {
            for field in variant_data.fields() {
                println!(" - {:?} {:?} ({:?})", field.ident.as_ref().unwrap(), ty_to_string(&field.ty), field.vis);
            }
        }
    };

    TokenStream::from_str("").unwrap()
}

fn ty_to_string(ty: &Ty) -> String {
    match *ty {
        Ty::Slice(ref inner) => format!("[{}]", ty_to_string(&inner)),
        Ty::Array(ref inner, ref length) => format!("[{}; {}]", ty_to_string(&inner), constexpr_to_string(length)),
        Ty::Ptr(ref inner) => format!("*{} {}", match inner.mutability {
                Mutability::Immutable => "const",
                Mutability::Mutable => "mut"
            }, ty_to_string(&inner.ty)
        ),
        Ty::Path(ref _self, ref path) => {
            let mut result = String::new();
            for segment in &path.segments {
                if !result.is_empty() { result += "::"; }
                result += &format!("{}", segment.ident);
            }
            result
        },
        _ => panic!("Not implemented: {:?}", ty)
    }
}

fn constexpr_to_string(expr: &ConstExpr) -> String {
    match *expr {
        _ => panic!("Not implemented: {:?}", expr)
    }
}
