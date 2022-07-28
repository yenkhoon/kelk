use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use std::str::FromStr;
use syn::{
    parse_macro_input, parse_quote, spanned::Spanned, Data, DeriveInput, Fields, FieldsNamed,
    FieldsUnnamed, GenericParam, Generics, Ident, Index, Type, TypeParamBound,
};

/// The attribute macro to inject the code at the beginning of entry functions
/// for the Wasm contract actor.
///
/// It can be added to the contract's instantiate, process and query functions
/// like this:
/// ```
/// use kelk::kelk_entry;
/// use kelk::context::Context;
///
/// type InstantiateMsg = ();
/// type ProcessMsg = ();
/// type QueryMsg = ();
///
/// enum Error {};
///
/// #[kelk_entry]
/// pub fn instantiate(ctx: Context, msg: InstantiateMsg) -> Result<(), Error> {
///    todo!();
/// }
///
/// #[kelk_entry]
/// pub fn process(ctx: Context, msg: ProcessMsg) -> Result<(), Error> {
///   todo!();
/// }
///
/// #[kelk_entry]
/// pub fn query(ctx: Context, msg: QueryMsg) -> Result<(), Error> {
///   todo!();
/// }
/// ```
///
/// where `InstantiateMsg`, `ProcessMsg`, and `QueryMsg` are contract defined
/// types that implement CBOR encoding.
#[proc_macro_attribute]
pub fn kelk_entry(
    _attr: proc_macro::TokenStream,
    mut item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let cloned = item.clone();
    let function = parse_macro_input!(cloned as syn::ItemFn);
    let name = function.sig.ident.to_string();

    let storage_method = match name.as_ref() {
        "instantiate" => "create",
        "process" => "load",
        "query" => "load",
        _ => {
            return proc_macro::TokenStream::from(quote! {
                compile_error!("entry function should be either \"instantiate\", \"process\", or \"query\""),
            })
        }
    };

    let gen_code = format!(
        r##"
        #[cfg(target_arch = "wasm32")]
        mod __wasm_export_{name} {{
            #[no_mangle]
            extern "C" fn {name}(msg_ptr: u64) -> u64 {{
                let ctx = kelk::context::OwnedContext {{
                    storage: kelk::storage::Storage::{method}(kelk::alloc::boxed::Box::new(kelk::Kelk::new()))
                        .unwrap(),
                    blockchain: kelk::blockchain::Blockchain::new(kelk::alloc::boxed::Box::new(
                        kelk::Kelk::new(),
                    )),
                }};

                kelk::do_{name}(&super::{name}, ctx.as_ref(), msg_ptr)
            }}
        }}
    "##,
        name = name,
        method = storage_method,
    );

    let entry = proc_macro::TokenStream::from_str(&gen_code).unwrap();
    item.extend(entry);
    item
}

#[proc_macro_derive(Codec)]
pub fn derive_codec(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree.
    let input = parse_macro_input!(input as DeriveInput);

    // Used in the quasi-quotation below as `#name`.
    let name = input.ident;

    // Add a bound `T: Codec` to every type parameter T.
    let generics = add_trait_bounds(input.generics, parse_quote!(Codec));
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Generate an expression to sum up the heap size of each field.
    let packed_len_body = packed_len_body(&input.data);
    let (to_bytes_body, from_bytes_body) = codec_body(&input.data);

    let expanded = quote! {
        impl #impl_generics Codec for #name #ty_generics #where_clause {

            const PACKED_LEN: usize = #packed_len_body;

            #[inline]
            fn to_bytes(&self, bytes: &mut [u8]) {
                debug_assert_eq!(bytes.len(), Self::PACKED_LEN);

                #to_bytes_body
            }

            #[inline]
            fn from_bytes(bytes: &[u8]) -> Self {
                debug_assert_eq!(bytes.len(), Self::PACKED_LEN);

                Self { #from_bytes_body }
            }
        }
    };

    //panic!("{}" , expanded.to_string());
    // Hand the output tokens back to the compiler.
    proc_macro::TokenStream::from(expanded)
}

fn packed_len_body(data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    // Expands to an expression like
                    //
                    //     0 + <self.x as Codec>::PACKED_LEN + <self.y as Codec>::PACKED_LEN
                    let recurse = fields.named.iter().map(|f| {
                        let ty = &f.ty;
                        quote_spanned! {f.span()=>
                            <#ty as Codec>::PACKED_LEN
                        }
                    });

                    quote! {
                        0  #(+ #recurse)*
                    }
                }
                Fields::Unnamed(ref fields) => {
                    // Expands to an expression like
                    //
                    //     0 + <self.0 as Codec>::PACKED_LEN + <self.1 as Codec>::PACKED_LEN
                    let recurse = fields.unnamed.iter().map(|f| {
                        let ty = &f.ty;
                        quote_spanned! {f.span()=>
                            <#ty as Codec>::PACKED_LEN
                        }
                    });
                    quote! {
                        0 #(+ #recurse)*
                    }
                }
                Fields::Unit => {
                    // Unit structs cannot own more than 0 bytes of heap memory.
                    quote!(0)
                }
            }
        }
        Data::Enum(_) | Data::Union(_) => unimplemented!(),
    }
}

// Add a bound `T: trait_bound` to every type parameter T.
fn add_trait_bounds(mut generics: Generics, trait_bound: TypeParamBound) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(trait_bound.clone());
        }
    }
    generics
}

fn codec_body(data: &Data) -> (TokenStream, TokenStream) {
    // this also contains `bytes` variable
    match *data {
        Data::Struct(ref data) => {
            match data.fields {
                //  Normal struct: named fields
                Fields::Named(FieldsNamed { ref named, .. }) => {
                    //  Collect references to all the field names. A precondition of
                    //  reaching this code path is that all fields HAVE names, so it
                    //  is safe to have an unreachable trap in the None condition.
                    let names: Vec<(&Type, &Ident)> = named
                        .iter()
                        .map(|f| (&f.ty, f.ident.as_ref().unwrap()))
                        .collect();
                    codegen_struct(&names)
                }
                //  Tuple struct: unnamed fields
                Fields::Unnamed(FieldsUnnamed { ref unnamed, .. }) => {
                    let mut nums: Vec<(&Type, Index)> = Vec::new();
                    for (i, f) in unnamed.into_iter().enumerate() {
                        nums.push((&f.ty, i.into()));
                    }
                    codegen_struct(&nums)
                }

                Fields::Unit => {
                    // Unit structs cannot own more than 0 bytes of heap memory.
                    (quote!(0), quote!(0))
                }
            }
        }
        Data::Enum(_) | Data::Union(_) => unimplemented!(),
    }
}

fn codegen_struct<T: ToTokens>(fields: &[(&Type, T)]) -> (TokenStream, TokenStream) {
    let mut beg_offset = quote! { 0 };
    let mut recurse_to_bytes = vec![];
    let mut recurse_from_bytes = vec![];

    for field in fields.iter() {
        let ty = field.0;
        let name = &field.1;
        let struct_size = quote! { <#ty as Codec>::PACKED_LEN };
        let end_offset = quote! { #beg_offset + #struct_size };
        let bytes_slice = quote! { bytes[#beg_offset..#end_offset] };

        recurse_to_bytes.push(quote! {
            Codec::to_bytes(&self.#name, &mut #bytes_slice);
        });

        recurse_from_bytes.push(quote! {
            #name: Codec::from_bytes(& #bytes_slice),
        });

        beg_offset = quote! { #beg_offset + #struct_size };
    }

    (
        quote! {
            #(#recurse_to_bytes)*
        },
        quote! {
            #(#recurse_from_bytes)*
        },
    )
}
