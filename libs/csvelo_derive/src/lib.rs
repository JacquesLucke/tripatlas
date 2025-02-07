use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, GenericParam, Ident, Lifetime};

/// Derives a CSV parser for a struct.
///
/// The struct is expected to have fields of the type `Option<Vec<T>>` whereby `T` has
/// to implement the `ParseCsvField` trait.
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
    let expanded = quote! {
        #header_struct
        #header_parser_function
        #chunk_parsing_function
        #reduced_function
        #parse_function
    };
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
}

fn parse_source_info(input: &DeriveInput) -> Result<SourceInfo, ()> {
    let mut csv_struct_fields = Vec::new();
    if let syn::Data::Struct(ref data) = input.data {
        if let syn::Fields::Named(ref fields) = data.fields {
            for field in fields.named.iter() {
                csv_struct_fields.push(ColumnFieldInfo {
                    name: field.ident.clone().ok_or(())?,
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

fn generate_header_struct(source_info: &SourceInfo) -> proc_macro2::TokenStream {
    let parts = source_info.csv_struct_fields.iter().map(|f| {
        let name = &f.name;
        quote! {
            #name: Option<usize>
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
        quote! {
            #name: header.get_column_index(stringify!(#name))
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
            quote! {
                #name: if let Some(column_i) = header.#name {
                    csvelo::parse_column_value(
                        records, column_i, |buffer| Ok(csvelo::ParseCsvField::parse_csv_field(buffer)?)).ok()
                } else {
                    None
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
            let mut #name = None;
        }
    });
    let process_parts = source_info.csv_struct_fields.iter().map(|f| {
        let name = &f.name;
        quote! {
            if let Some(_) = header.#name {
                s.spawn(|_| {
                    let mut value_slices = vec![];
                    for chunk in &chunks {
                        if let Some(chunk) = chunk.#name.as_ref() {
                            value_slices.push(chunk.as_slice());
                        }
                        else {
                            return;
                        }
                    }
                    #name = Some(csvelo::flatten_slices(&value_slices));
                });
            }
        }
    });
    let output_parts = source_info.csv_struct_fields.iter().map(|f| {
        let name = &f.name;
        quote! {
            #name
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
            fn from_csv_buffer<'buffer>(buffer: &'buffer [u8])  -> std::result::Result<(Self, usize), ()> #buffer_lifetimes_bound {
                let sections = csvelo::split_header_and_data(buffer);
                let header = csvelo::parse_header(sections.header);
                let header = #header_name::from_header_chunk(header)?;
                let data_chunks = csvelo::split_csv_buffer_into_record_aligned_chunks(sections.data, 256 * 1024);
                let parsed_chunks = data_chunks.par_iter().map(|chunk| {
                    let records = csvelo::CsvRecords::from_buffer(chunk);
                    let size = records.len();
                    match #main_name::parse_csv_chunk(&header, &records) {
                        Ok(parsed_chunk) => Ok((parsed_chunk, size)),
                        Err(err) => Err(err),
                    }
                }).collect::<std::result::Result<Vec<_>, _>>()?;
                let records_num = parsed_chunks.iter().map(|(_, size)| *size).sum::<usize>();
                let parsed_chunks = parsed_chunks.into_iter().map(|(chunk, _)| chunk).collect();
                match #main_name::from_csv_parse_chunks(&header, parsed_chunks) {
                    Ok(parsed) => Ok((parsed, records_num)),
                    Err(err) => Err(err),
                }
            }
        }
    }
}
