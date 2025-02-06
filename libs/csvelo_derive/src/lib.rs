use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, GenericParam, Ident, Lifetime, Type};

#[proc_macro_derive(CSVParser)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);

    let source_info = match parse_source_info(&input) {
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

struct SourceInfo<'a> {
    main_name: Ident,
    header_name: Ident,
    csv_struct_fields: Vec<ColumnFieldInfo>,
    input: &'a DeriveInput,
    buffer_lifetimes_bound: proc_macro2::TokenStream,
}

#[derive(Debug)]
struct ColumnFieldInfo {
    name: Ident,
    optional: bool,
    // field_type_name: Ident,
}

fn parse_source_info(input: &DeriveInput) -> Result<SourceInfo, ()> {
    let mut csv_struct_fields = Vec::new();
    if let syn::Data::Struct(ref data) = input.data {
        if let syn::Fields::Named(ref fields) = data.fields {
            for field in fields.named.iter() {
                let option_seg = get_first_path_segment_with_name(&field.ty, "Option");
                csv_struct_fields.push(ColumnFieldInfo {
                    name: field.ident.clone().ok_or(())?,
                    optional: option_seg.is_some(),
                });
            }
        } else {
            return Err(());
        }
    } else {
        return Err(());
    }
    let header_name = Ident::new(&format!("{}CsvHeader", &input.ident), Span::call_site());
    let lifetimes: Vec<&Lifetime> = input
        .generics
        .params
        .iter()
        .filter_map(|param| {
            if let GenericParam::Lifetime(lifetime) = param {
                Some(&lifetime.lifetime)
            } else {
                None
            }
        })
        .collect();
    let buffer_lifetimes_bound = if lifetimes.is_empty() {
        quote! {}
    } else {
        quote! { where 'buffer: #(#lifetimes)+* }
    };
    Ok(SourceInfo {
        main_name: input.ident.clone(),
        header_name: header_name,
        csv_struct_fields,
        input,
        buffer_lifetimes_bound: buffer_lifetimes_bound,
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
    let (impl_generics, ty_generics, where_clause) = source_info.input.generics.split_for_impl();
    let buffer_lifetimes_bound = &source_info.buffer_lifetimes_bound;
    quote! {
        impl #impl_generics #main_name #ty_generics #where_clause {
            fn parse_csv_chunk<'buffer>(
                header: &#header_name,
                records: &csvelo::CsvRecords<'buffer>,
            ) -> std::result::Result<Self, ()> #buffer_lifetimes_bound {
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

    let setup_parts = source_info.csv_struct_fields.iter().map(|f| {
        let name = &f.name;
        quote! {
            let mut #name = vec![];
        }
    });
    let process_parts = source_info.csv_struct_fields.iter().map(|f| {
        let name = &f.name;
        if f.optional {
            quote! {
                if let Some(_) = header.#name {
                    s.spawn(|_| {
                        let mut value_slices = vec![];
                        for chunk in &chunks {
                            value_slices.push(chunk.#name.as_ref().unwrap().as_slice());
                        }
                        #name = csvelo::flatten_slices(&value_slices);
                    });
                }
            }
        } else {
            quote! {
                s.spawn(|_| {
                    let mut value_slices = vec![];
                    for chunk in &chunks {
                        value_slices.push(chunk.#name.as_slice());
                    }
                    #name = csvelo::flatten_slices(&value_slices);
                });
            }
        }
    });
    let output_parts = source_info.csv_struct_fields.iter().map(|f| {
        let name = &f.name;
        if f.optional {
            quote! {
                #name: if let Some(_) = header.#name { Some(#name) } else { None }
            }
        } else {
            quote! {
                #name
            }
        }
    });

    let (impl_generics, ty_generics, where_clause) = source_info.input.generics.split_for_impl();
    quote! {
        impl #impl_generics #main_name #ty_generics #where_clause {
            fn from_csv_parse_chunks(header: &#header_name, chunks: Vec<Self>) -> std::result::Result<Self, ()> {
                #(#setup_parts)*
                rayon::scope(|s| {
                    #(#process_parts)*
                });
                Ok(Self {
                    #(#output_parts),*
                })
            }
        }
    }
}

fn generate_full_parse_function(source_info: &SourceInfo) -> proc_macro2::TokenStream {
    let header_name = &source_info.header_name;
    let main_name = &source_info.main_name;
    let (impl_generics, ty_generics, where_clause) = source_info.input.generics.split_for_impl();
    let buffer_lifetimes_bound = &source_info.buffer_lifetimes_bound;
    quote! {
        impl #impl_generics #main_name #ty_generics #where_clause {
            fn from_csv_buffer<'buffer>(buffer: &'buffer [u8])  -> std::result::Result<Self, ()> #buffer_lifetimes_bound {
                let sections = csvelo::split_header_and_data(buffer);
                let header = csvelo::parse_header(sections.header);
                let header = #header_name::from_header_chunk(header)?;
                let data_chunks = csvelo::split_csv_buffer_into_record_aligned_chunks(sections.data, 256 * 1024);
                let parsed_chunks = data_chunks.par_iter().map(|chunk| {
                    let records = csvelo::CsvRecords::from_buffer(chunk);
                    #main_name::parse_csv_chunk(&header, &records)
                }).collect::<std::result::Result<Vec<_>, _>>()?;
                #main_name::from_csv_parse_chunks(&header, parsed_chunks)
            }
        }
    }
}
