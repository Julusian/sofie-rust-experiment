#![recursion_limit = "128"]

use syn::Field;

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

#[proc_macro_derive(ProtectedId, attributes(protected_value))]
pub fn protected_id_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Construct a string representation of the type definition
    let s = input.to_string();

    // Parse the string representation
    let ast = syn::parse_derive_input(&s).unwrap();

    // Build the impl
    let gen = generate_impl(&ast);

    // Return the generated impl
    gen.parse().unwrap()
}

fn generate_impl(ast: &syn::DeriveInput) -> quote::Tokens {
    let ident = &ast.ident;
    let generics = &ast.generics;
    let where_clause = &ast.generics.where_clause;

    let ident_protected_value = get_ident_protected_value(&ast.body);

    quote! {
        impl #ident #generics #where_clause {
            pub fn new () -> Self {
                #ident {
                    #ident_protected_value: uuid::Uuid::new_v4().to_string()
                }
            }

            pub fn new_from (id: String) -> Self {
                #ident {
                    #ident_protected_value: id
                }
            }
        }
        impl IProtectedId for #ident {
            pub fn unprotect (&self) -> String {
                self.id.clone()
            }
        }
    }
}

fn get_ident_protected_value(body: &syn::Body) -> &syn::Ident {
    match *body {
        syn::Body::Enum(_) => panic!("ProtectedId cannot be implemented for enums"),
        syn::Body::Struct(syn::VariantData::Unit) => {
            panic!("ProtectedId cannot be implemented for Unit structs")
        }
        syn::Body::Struct(syn::VariantData::Tuple(_)) => {
            panic!("ProtectedId cannot be implemented for Tuple structs")
        }
        syn::Body::Struct(syn::VariantData::Struct(ref s)) => {
            let field = s.iter().find(is_protected_value).unwrap_or_else(|| {
                panic!("Struct does not have a field with attribute `protected_value`")
            });
            field.ident.as_ref().unwrap_or_else(|| {
                panic!("Cannot find identifier for field marked with `protected_value` attribute")
            })
        }
    }
}

fn is_protected_value(field: &&Field) -> bool {
    field
        .attrs
        .iter()
        .any(|a| a.value.name() == "protected_value")
}
