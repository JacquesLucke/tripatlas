use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident, Type};

#[proc_macro_derive(CSVParser)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);

    let source_info = match parse_source_info(input.clone()) {
        Ok(source_info) => source_info,
        Err(_) => panic!(),
    };

    let header_struct = generate_header_struct(&source_info);
    let header_parser_function = generate_header_parser_function(&source_info);
    let chunk_parsing_function = generate_chunk_parsing_function(&source_info);
    let reduced_function = generate_reduce_function(&source_info);
    let parse_function = generate_full_parse_function(&source_info);
    let expanded = quote! { #header_struct #header_parser_function #chunk_parsing_function #reduced_function #parse_function};
    expanded.into()
}

#[derive(Debug)]
struct SourceInfo {
    main_name: Ident,
    header_name: Ident,
    csv_struct_fields: Vec<ColumnFieldInfo>,
}

#[derive(Debug)]
struct ColumnFieldInfo {
    name: Ident,
    optional: bool,
    // field_type_name: Ident,
}

fn parse_source_info(input: DeriveInput) -> Result<SourceInfo, ()> {
    let mut csv_struct_fields = Vec::new();
    if let syn::Data::Struct(ref data) = input.data {
        if let syn::Fields::Named(ref fields) = data.fields {
            for field in fields.named.iter() {
                let option_seg = get_first_path_segment_with_name(&field.ty, "Option");
                // let vec_seg = if let Some(option_seg) = option_seg {
                //     get_first_path_segment_with_name(
                //         get_first_inner_type(option_seg).ok_or(())?,
                //         "Vec",
                //     )
                //     .ok_or(())?
                // } else {
                //     get_first_path_segment_with_name(&field.ty, "Vec").ok_or(())?
                // };
                // let field_ty = get_first_inner_type(vec_seg).ok_or(())?;
                csv_struct_fields.push(ColumnFieldInfo {
                    name: field.ident.clone().ok_or(())?,
                    optional: option_seg.is_some(),
                    // field_type_name: if let Type::Path(path_ty) = field_ty {
                    //     path_ty.path.segments.last().unwrap().ident.clone()
                    // } else {
                    //     return Err(());
                    // },
                });
            }
        } else {
            return Err(());
        }
    } else {
        return Err(());
    }
    let header_name = Ident::new(&format!("{}CsvHeader", &input.ident), Span::call_site());
    Ok(SourceInfo {
        main_name: input.ident,
        header_name: header_name,
        csv_struct_fields,
    })
}

fn get_first_path_segment_with_name<'a>(
    ty: &'a syn::Type,
    name: &str,
) -> Option<&'a syn::PathSegment> {
    if let Type::Path(path_ty) = ty {
        path_ty.path.segments.iter().find(|s| s.ident == name)
    } else {
        None
    }
}

fn get_first_inner_type(outer: &syn::PathSegment) -> Option<&syn::Type> {
    if let syn::PathArguments::AngleBracketed(args) = &outer.arguments {
        if let Some(arg) = args.args.first() {
            if let syn::GenericArgument::Type(inner_ty) = arg {
                return Some(inner_ty);
            }
        }
    }
    None
}

fn generate_header_struct(source_info: &SourceInfo) -> proc_macro2::TokenStream {
    let parts = source_info.csv_struct_fields.iter().map(|f| {
        let name = &f.name;
        if f.optional {
            quote! {
                #name: Option<usize>
            }
        } else {
            quote! {
                #name: usize
            }
        }
    });
    let header_name = &source_info.header_name;
    quote! {
        struct #header_name {
            #(#parts),*
        }
    }
}

fn generate_header_parser_function(source_info: &SourceInfo) -> proc_macro2::TokenStream {
    let header_name = &source_info.header_name;
    let parts = source_info.csv_struct_fields.iter().map(|f| {
        let name = &f.name;
        if f.optional {
            quote! {
                #name: header.get_column_index(stringify!(#name))
            }
        } else {
            quote! {
                #name: header.get_column_index(stringify!(#name)).ok_or(())?
            }
        }
    });
    quote! {
        impl #header_name {
            fn from_header_chunk(header: csvelo::CsvHeader) -> std::result::Result<Self, ()> {
                Ok(Self {
                    #(#parts),*
                })
            }
        }
    }
}

fn generate_chunk_parsing_function(source_info: &SourceInfo) -> proc_macro2::TokenStream {
    let header_name = &source_info.header_name;
    let main_name = &source_info.main_name;
    let parts = source_info.csv_struct_fields.iter().map(|f| {
        let name = &f.name;
        if f.optional {
            quote! {
                #name: if let Some(column_i) = header.#name {
                    csvelo::parse_column_value(
                        records, column_i, |buffer| Ok(csvelo::ParseCsvField::parse_csv_field(buffer)?)).ok()
                } else {
                    None
                }
            }
        } else {
            quote! {
                #name: csvelo::parse_column_value(
                    records, header.#name, |buffer| Ok(csvelo::ParseCsvField::parse_csv_field(buffer)?))?
            }
        }
    });
    quote! {
        impl #main_name {
            fn parse_csv_chunk<'a>(
                header: &#header_name,
                records: &csvelo::CsvRecords<'a>,
            ) -> std::result::Result<Self, ()> {
                Ok(Self {
                    #(#parts),*
                })
            }
        }
    }
}

fn generate_reduce_function(source_info: &SourceInfo) -> proc_macro2::TokenStream {
    let header_name = &source_info.header_name;
    let main_name = &source_info.main_name;
    let parts = source_info.csv_struct_fields.iter().map(|f| {
        let name = &f.name;
        if f.optional {
            quote! {
                #name: if let Some(_) = header.#name {
                    Some(
                        chunks
                            .iter()
                            .flat_map(|chunk| chunk.#name.clone().unwrap())
                            .collect(),
                    )
                } else {
                    None
                }
            }
        } else {
            quote! {
                #name: chunks.iter().flat_map(|chunk| chunk.#name.clone()).collect()
            }
        }
    });
    quote! {
        impl #main_name {
            fn from_csv_parse_chunks(header: &#header_name, chunks: Vec<Self>) -> std::result::Result<Self, ()> {
                Ok(Self {
                    #(#parts),*
                })
            }
        }
    }
}

fn generate_full_parse_function(source_info: &SourceInfo) -> proc_macro2::TokenStream {
    let header_name = &source_info.header_name;
    let main_name = &source_info.main_name;
    quote! {
        impl #main_name {
            fn from_csv_buffer(buffer: &[u8]) -> std::result::Result<Self, ()> {
                let sections = csvelo::split_header_and_data(buffer);
                let header = csvelo::parse_header(sections.header);
                let header = #header_name::from_header_chunk(header)?;
                let data_chunks = csvelo::split_csv_buffer_into_record_aligned_chunks(sections.data, 256 * 1024);
                let mut parsed_chunks = vec![];
                for chunk in data_chunks {
                    let records = csvelo::CsvRecords::from_buffer(chunk);
                    parsed_chunks.push(#main_name::parse_csv_chunk(&header, &records)?);
                }
                #main_name::from_csv_parse_chunks(&header, parsed_chunks)
            }
        }
    }
}
