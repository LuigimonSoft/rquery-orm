use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Lit, Meta, NestedMeta};

#[proc_macro_derive(Entity, attributes(table, column, key, relation))]
pub fn entity(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;

    // table attributes
    let mut table_name = struct_name.to_string();
    let mut table_schema: Option<String> = None;
    for attr in &input.attrs {
        if attr.path.is_ident("table") {
            if let Ok(Meta::List(list)) = attr.parse_meta() {
                for nested in list.nested.iter() {
                    match nested {
                        NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("name") => {
                            if let Lit::Str(s) = &nv.lit {
                                table_name = s.value();
                            }
                        }
                        NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("schema") => {
                            if let Lit::Str(s) = &nv.lit {
                                table_schema = Some(s.value());
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    let mut columns = Vec::new();
    let mut keys = Vec::new();
    let mut relations = Vec::new();
    let mut from_ms_fields = Vec::new();
    let mut from_pg_fields = Vec::new();
    let mut from_ms_fields_with_prefix = Vec::new();
    let mut from_pg_fields_with_prefix = Vec::new();
    let mut assoc_consts = Vec::new();
    let mut insert_stmts = Vec::new();
    let mut update_set_stmts = Vec::new();
    let mut update_where_stmts = Vec::new();
    let mut delete_where_stmts = Vec::new();
    let mut validate_stmts = Vec::new();
    let mut first_key_col = String::new();
    let mut has_identity = false;
    let mut key_trait_impls = Vec::new();

    if let Data::Struct(ds) = input.data {
        if let Fields::Named(fields_named) = ds.fields {
            for field in fields_named.named {
                let ident = field.ident.unwrap();
                let ty = field.ty.clone();

                let mut is_option = false;
                let mut is_string = false;
                let mut inner_ty = ty.clone();
                if let syn::Type::Path(tp) = &ty {
                    if tp.path.segments.len() == 1 && tp.path.segments[0].ident == "Option" {
                        is_option = true;
                        if let syn::PathArguments::AngleBracketed(args) = &tp.path.segments[0].arguments {
                            if let Some(syn::GenericArgument::Type(t)) = args.args.first() {
                                inner_ty = t.clone();
                                if let syn::Type::Path(itp) = t {
                                    if itp.path.is_ident("String") {
                                        is_string = true;
                                    }
                                }
                            }
                        }
                    } else if tp.path.is_ident("String") {
                        is_string = true;
                    }
                }

                // relation handling
                let mut is_relation = false;
                let mut rel_foreign_key = String::new();
                let mut rel_table = String::new();
                let mut rel_table_number: Option<u32> = None;
                let mut rel_ignore_in_update = false;
                let mut rel_ignore_in_insert = false;

                for attr in field.attrs.iter() {
                    if attr.path.is_ident("relation") {
                        is_relation = true;
                        if let Ok(Meta::List(list)) = attr.parse_meta() {
                            for nested in list.nested.iter() {
                                match nested {
                                    NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("foreign_key") => {
                                        if let Lit::Str(s) = &nv.lit {
                                            rel_foreign_key = s.value();
                                        }
                                    }
                                    NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("table") => {
                                        if let Lit::Str(s) = &nv.lit {
                                            rel_table = s.value();
                                        }
                                    }
                                    NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("table_number") => {
                                        if let Lit::Int(i) = &nv.lit {
                                            rel_table_number = i.base10_parse().ok();
                                        }
                                    }
                                    NestedMeta::Meta(Meta::Path(p)) if p.is_ident("ignore_in_update") => {
                                        rel_ignore_in_update = true;
                                    }
                                    NestedMeta::Meta(Meta::Path(p)) if p.is_ident("ignore_in_insert") => {
                                        rel_ignore_in_insert = true;
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }

                if is_relation {
                    let table_num_tokens = match rel_table_number {
                        Some(v) => quote! { Some(#v) },
                        None => quote! { None },
                    };
                    relations.push(quote! {
                        ::rquery_orm::mapping::RelationMeta {
                            name: stringify!(#ident),
                            foreign_key: #rel_foreign_key,
                            table: #rel_table,
                            table_number: #table_num_tokens,
                            ignore_in_update: #rel_ignore_in_update,
                            ignore_in_insert: #rel_ignore_in_insert,
                        }
                    });
                    continue;
                }

                // column and key attributes
                let mut col_name = ident.to_string();
                let mut col_name_lit: Option<syn::LitStr> = None;
                let mut is_key = false;
                let mut is_identity = false;
                let mut required = false;
                let mut allow_null = false;
                let mut max_length: Option<usize> = None;
                let mut min_length: Option<usize> = None;
                let mut allow_empty = true;
                let mut regex: Option<String> = None;
                let mut err_max_length: Option<String> = None;
                let mut err_min_length: Option<String> = None;
                let mut err_required: Option<String> = None;
                let mut err_allow_null: Option<String> = None;
                let mut err_allow_empty: Option<String> = None;
                let mut err_regex: Option<String> = None;
                let mut ignore = false;
                let mut ignore_in_update = false;
                let mut ignore_in_insert = false;
                let mut ignore_in_delete = false;
                let mut key_ignore_in_update = false;
                let mut key_ignore_in_insert = false;

                for attr in field.attrs.iter() {
                    if attr.path.is_ident("column") {
                        if let Ok(Meta::List(list)) = attr.parse_meta() {
                            for nested in list.nested.iter() {
                                match nested {
                                    NestedMeta::Meta(Meta::NameValue(nv)) => {
                                        if nv.path.is_ident("name") {
                                            if let Lit::Str(s) = &nv.lit { col_name = s.value(); }
                                        } else if nv.path.is_ident("max_length") {
                                            if let Lit::Int(i) = &nv.lit { max_length = i.base10_parse().ok(); }
                                        } else if nv.path.is_ident("min_length") {
                                            if let Lit::Int(i) = &nv.lit { min_length = i.base10_parse().ok(); }
                                        } else if nv.path.is_ident("regex") {
                                            if let Lit::Str(s) = &nv.lit { regex = Some(s.value()); }
                                        } else if nv.path.is_ident("error_max_length") {
                                            if let Lit::Str(s) = &nv.lit { err_max_length = Some(s.value()); }
                                        } else if nv.path.is_ident("error_min_length") {
                                            if let Lit::Str(s) = &nv.lit { err_min_length = Some(s.value()); }
                                        } else if nv.path.is_ident("error_required") {
                                            if let Lit::Str(s) = &nv.lit { err_required = Some(s.value()); }
                                        } else if nv.path.is_ident("error_allow_null") {
                                            if let Lit::Str(s) = &nv.lit { err_allow_null = Some(s.value()); }
                                        } else if nv.path.is_ident("error_allow_empty") {
                                            if let Lit::Str(s) = &nv.lit { err_allow_empty = Some(s.value()); }
                                        } else if nv.path.is_ident("error_regex") {
                                            if let Lit::Str(s) = &nv.lit { err_regex = Some(s.value()); }
                                        } else if nv.path.is_ident("allow_empty") {
                                            if let Lit::Bool(b) = &nv.lit { allow_empty = b.value; }
                                        } else if nv.path.is_ident("required") {
                                            if let Lit::Bool(b) = &nv.lit { required = b.value; }
                                        } else if nv.path.is_ident("allow_null") {
                                            if let Lit::Bool(b) = &nv.lit { allow_null = b.value; }
                                        } else if nv.path.is_ident("ignore_in_update") {
                                            if let Lit::Bool(b) = &nv.lit { ignore_in_update = b.value; }
                                        } else if nv.path.is_ident("ignore_in_insert") {
                                            if let Lit::Bool(b) = &nv.lit { ignore_in_insert = b.value; }
                                        } else if nv.path.is_ident("ignore_in_delete") {
                                            if let Lit::Bool(b) = &nv.lit { ignore_in_delete = b.value; }
                                        } else if nv.path.is_ident("ignore") {
                                            if let Lit::Bool(b) = &nv.lit { ignore = b.value; }
                                        }
                                    }
                                    NestedMeta::Meta(Meta::Path(p)) => {
                                        if p.is_ident("required") { required = true; }
                                        else if p.is_ident("allow_null") { allow_null = true; }
                                        else if p.is_ident("allow_empty") { allow_empty = true; }
                                        else if p.is_ident("ignore") { ignore = true; }
                                        else if p.is_ident("ignore_in_update") { ignore_in_update = true; }
                                        else if p.is_ident("ignore_in_insert") { ignore_in_insert = true; }
                                        else if p.is_ident("ignore_in_delete") { ignore_in_delete = true; }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    } else if attr.path.is_ident("key") {
                        is_key = true;
                        if let Ok(Meta::List(list)) = attr.parse_meta() {
                            for nested in list.nested.iter() {
                                match nested {
                                    NestedMeta::Meta(Meta::NameValue(nv)) => {
                                        if nv.path.is_ident("is_identity") {
                                            if let Lit::Bool(b) = &nv.lit { is_identity = b.value; }
                                        } else if nv.path.is_ident("name") {
                                            if let Lit::Str(s) = &nv.lit { col_name = s.value(); }
                                        } else if nv.path.is_ident("ignore_in_update") {
                                            if let Lit::Bool(b) = &nv.lit { key_ignore_in_update = b.value; }
                                        } else if nv.path.is_ident("ignore_in_insert") {
                                            if let Lit::Bool(b) = &nv.lit { key_ignore_in_insert = b.value; }
                                        }
                                    }
                                    NestedMeta::Meta(Meta::Path(p)) => {
                                        if p.is_ident("ignore_in_update") { key_ignore_in_update = true; }
                                        else if p.is_ident("ignore_in_insert") { key_ignore_in_insert = true; }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }

                // literal for column name token
                let col_name_lit_inner = syn::LitStr::new(&col_name, proc_macro2::Span::call_site());

                let max_length_token = match max_length { Some(v) => quote! { Some(#v) }, None => quote! { None } };
                let min_length_token = match min_length { Some(v) => quote! { Some(#v) }, None => quote! { None } };
                let regex_token = match regex.as_ref() { Some(s) => quote! { Some(#s) }, None => quote! { None } };
                let err_max_length_token = match err_max_length.as_ref() { Some(s) => quote! { Some(#s) }, None => quote! { None } };
                let err_min_length_token = match err_min_length.as_ref() { Some(s) => quote! { Some(#s) }, None => quote! { None } };
                let err_required_token = match err_required.as_ref() { Some(s) => quote! { Some(#s) }, None => quote! { None } };
                let err_allow_null_token = match err_allow_null.as_ref() { Some(s) => quote! { Some(#s) }, None => quote! { None } };
                let err_allow_empty_token = match err_allow_empty.as_ref() { Some(s) => quote! { Some(#s) }, None => quote! { None } };
                let err_regex_token = match err_regex.as_ref() { Some(s) => quote! { Some(#s) }, None => quote! { None } };

                columns.push(quote! {
                    ::rquery_orm::mapping::ColumnMeta {
                        name: #col_name_lit_inner,
                        required: #required,
                        allow_null: #allow_null,
                        max_length: #max_length_token,
                        min_length: #min_length_token,
                        allow_empty: #allow_empty,
                        regex: #regex_token,
                        error_max_length: #err_max_length_token,
                        error_min_length: #err_min_length_token,
                        error_required: #err_required_token,
                        error_allow_null: #err_allow_null_token,
                        error_allow_empty: #err_allow_empty_token,
                        error_regex: #err_regex_token,
                        ignore: #ignore,
                        ignore_in_update: #ignore_in_update,
                        ignore_in_insert: #ignore_in_insert,
                        ignore_in_delete: #ignore_in_delete,
                    }
                });

                if is_key {
                    keys.push(quote! {
                        ::rquery_orm::mapping::KeyMeta {
                            column: #col_name,
                            is_identity: #is_identity,
                            ignore_in_update: #key_ignore_in_update,
                            ignore_in_insert: #key_ignore_in_insert,
                        }
                    });
                    if first_key_col.is_empty() {
                        first_key_col = col_name.clone();
                        if let syn::Type::Path(tp) = &ty {
                            if tp.qself.is_none() {
                                if tp.path.is_ident("i32") {
                                    key_trait_impls.push(quote! { impl ::rquery_orm::mapping::KeyAsInt for #struct_name { fn key(&self) -> i32 { self.#ident } } });
                                } else if tp.path.is_ident("String") {
                                    key_trait_impls.push(quote! { impl ::rquery_orm::mapping::KeyAsString for #struct_name { fn key(&self) -> String { self.#ident.clone() } } });
                                } else {
                                    let last = tp.path.segments.last().unwrap().ident.to_string();
                                    if last == "Uuid" {
                                        key_trait_impls.push(quote! { impl ::rquery_orm::mapping::KeyAsGuid for #struct_name { fn key(&self) -> uuid::Uuid { self.#ident } } });
                                    }
                                }
                            }
                        }
                    }
                    if is_identity { has_identity = true; }
                }

                // push associated const for this column
                assoc_consts.push(quote! { pub const #ident: &'static str = #col_name_lit_inner; });

                if is_option {
                    if is_string {
                        from_ms_fields.push(quote! { #ident: row.try_get::<&str, _>(#col_name_lit_inner)?.map(|v| v.to_string()) });
                        from_ms_fields_with_prefix.push(quote! { #ident: { let k = format!("{}_{}", prefix, #col_name_lit_inner); row.try_get::<&str, _>(k.as_str())?.map(|v| v.to_string()) } });
                    } else {
                        from_ms_fields.push(quote! { #ident: row.try_get::<#inner_ty, _>(#col_name_lit_inner)? });
                        from_ms_fields_with_prefix.push(quote! { #ident: { let k = format!("{}_{}", prefix, #col_name_lit_inner); row.try_get::<#inner_ty, _>(k.as_str())? } });
                    }
                    from_pg_fields.push(quote! { #ident: row.try_get(#col_name_lit_inner)? });
                    from_pg_fields_with_prefix.push(quote! { #ident: { let k = format!("{}_{}", prefix, #col_name_lit_inner); row.try_get(k.as_str())? } });
                } else {
                    if is_string {
                        from_ms_fields.push(quote! { #ident: row.try_get::<&str, _>(#col_name_lit_inner)?.unwrap().to_string() });
                        from_ms_fields_with_prefix.push(quote! { #ident: { let k = format!("{}_{}", prefix, #col_name_lit_inner); row.try_get::<&str, _>(k.as_str())?.unwrap().to_string() } });
                    } else {
                        from_ms_fields.push(quote! { #ident: row.try_get::<#inner_ty, _>(#col_name_lit_inner)?.unwrap() });
                        from_ms_fields_with_prefix.push(quote! { #ident: { let k = format!("{}_{}", prefix, #col_name_lit_inner); row.try_get::<#inner_ty, _>(k.as_str())?.unwrap() } });
                    }
                    from_pg_fields.push(quote! { #ident: row.try_get(#col_name_lit_inner)? });
                    from_pg_fields_with_prefix.push(quote! { #ident: { let k = format!("{}_{}", prefix, #col_name_lit_inner); row.try_get(k.as_str())? } });
                }

                if !is_identity && !ignore && !ignore_in_insert && !key_ignore_in_insert {
                    insert_stmts.push(quote! {
                        cols.push(#col_name);
                        vals.push(match style {
                            ::rquery_orm::query::PlaceholderStyle::AtP => format!("@P{}", idx),
                            ::rquery_orm::query::PlaceholderStyle::Dollar => format!("${}", idx),
                        });
                        params.push(self.#ident.clone().to_param());
                        idx += 1;
                    });
                }

                if !is_key {
                    if !ignore && !ignore_in_update {
                        update_set_stmts.push(quote! {
                            sets.push(format!("{} = {}", #col_name, match style {
                                ::rquery_orm::query::PlaceholderStyle::AtP => format!("@P{}", idx),
                                ::rquery_orm::query::PlaceholderStyle::Dollar => format!("${}", idx),
                            }));
                            params.push(self.#ident.clone().to_param());
                            idx += 1;
                        });
                    }
                } else if !key_ignore_in_update {
                    update_where_stmts.push(quote! {
                        wheres.push(format!("{} = {}", #col_name, match style {
                            ::rquery_orm::query::PlaceholderStyle::AtP => format!("@P{}", idx),
                            ::rquery_orm::query::PlaceholderStyle::Dollar => format!("${}", idx),
                        }));
                        params.push(self.#ident.clone().to_param());
                        idx += 1;
                    });
                    delete_where_stmts.push(quote! {
                        wheres.push(format!("{} = {}", #col_name, match style {
                            ::rquery_orm::query::PlaceholderStyle::AtP => format!("@P{}", idx),
                            ::rquery_orm::query::PlaceholderStyle::Dollar => format!("${}", idx),
                        }));
                        params.push(self.#ident.clone().to_param());
                        idx += 1;
                    });
                }

                // validation generation
                let field_literal = col_name.clone();
                let required_push = if let Some(msg) = err_required.clone() {
                    quote! { errors.push(#msg.to_string()); }
                } else {
                    let n = field_literal.clone();
                    quote! { errors.push(format!("{} is required", #n)); }
                };
                let allow_null_push = if let Some(msg) = err_allow_null.clone() {
                    quote! { errors.push(#msg.to_string()); }
                } else {
                    let n = field_literal.clone();
                    quote! { errors.push(format!("{} cannot be null", #n)); }
                };
                let allow_empty_push = if let Some(msg) = err_allow_empty.clone() {
                    quote! { errors.push(#msg.to_string()); }
                } else {
                    let n = field_literal.clone();
                    quote! { errors.push(format!("{} cannot be empty", #n)); }
                };
                let max_check = if let Some(max) = max_length {
                    let err_push = if let Some(msg) = err_max_length.clone() {
                        quote! { errors.push(#msg.to_string()); }
                    } else {
                        let n = field_literal.clone();
                        quote! { errors.push(format!("{} exceeds max length {}", #n, #max)); }
                    };
                    quote! { if value.len() > #max { #err_push } }
                } else { quote! {} };
                let min_check = if let Some(min) = min_length {
                    let err_push = if let Some(msg) = err_min_length.clone() {
                        quote! { errors.push(#msg.to_string()); }
                    } else {
                        let n = field_literal.clone();
                        quote! { errors.push(format!("{} below min length {}", #n, #min)); }
                    };
                    quote! { if value.len() < #min { #err_push } }
                } else { quote! {} };
                let regex_check = if let Some(re) = regex.clone() {
                    let err_push = if let Some(msg) = err_regex.clone() {
                        quote! { errors.push(#msg.to_string()); }
                    } else {
                        let n = field_literal.clone();
                        quote! { errors.push(format!("{} has invalid format", #n)); }
                    };
                    quote! {{
                        static RE: ::std::sync::OnceLock<::regex::Regex> = ::std::sync::OnceLock::new();
                        let re = RE.get_or_init(|| ::regex::Regex::new(#re).unwrap());
                        if !re.is_match(value) { #err_push }
                    }}
                } else { quote! {} };
                if is_option {
                    let some_branch = if is_string {
                        quote! {
                            if value.is_empty() {
                                if #required {
                                    #required_push
                                } else if !#allow_empty {
                                    #allow_empty_push
                                }
                            }
                            #max_check
                            #min_check
                            #regex_check
                        }
                    } else {
                        quote! {}
                    };
                    validate_stmts.push(quote! {
                        match &self.#ident {
                            None => {
                                if #required {
                                    #required_push
                                } else if !#allow_null {
                                    #allow_null_push
                                }
                            }
                            Some(value) => { #some_branch }
                        }
                    });
                } else if is_string {
                    validate_stmts.push(quote! {
                        let value = &self.#ident;
                        if value.is_empty() {
                            if #required {
                                #required_push
                            } else if !#allow_empty {
                                #allow_empty_push
                            }
                        }
                        #max_check
                        #min_check
                        #regex_check
                    });
                }
            }
        }
    }

    let table_name_lit = syn::LitStr::new(&table_name, proc_macro2::Span::call_site());
    let schema_tokens = match table_schema {
        Some(s) => {
            let s_lit = syn::LitStr::new(&s, proc_macro2::Span::call_site());
            quote! { Some(#s_lit) }
        }
        None => quote! { None },
    };
    let first_key_col_literal = first_key_col.clone();

    let expanded = quote! {
        const COLUMNS: &[::rquery_orm::mapping::ColumnMeta] = &[#(#columns),*];
        const KEYS: &[::rquery_orm::mapping::KeyMeta] = &[#(#keys),*];
        const RELATIONS: &[::rquery_orm::mapping::RelationMeta] = &[#(#relations),*];
        static TABLE_META: ::rquery_orm::mapping::TableMeta = ::rquery_orm::mapping::TableMeta {
            name: #table_name_lit,
            schema: #schema_tokens,
            columns: COLUMNS,
            keys: KEYS,
            relations: RELATIONS,
        };

        impl ::rquery_orm::mapping::Entity for #struct_name {
            fn table() -> &'static ::rquery_orm::mapping::TableMeta {
                &TABLE_META
            }
        }

        impl ::rquery_orm::mapping::FromRowNamed for #struct_name {
            fn from_row_ms(row: &tiberius::Row) -> anyhow::Result<Self> {
                Ok(Self { #(#from_ms_fields),* })
            }
            fn from_row_pg(row: &tokio_postgres::Row) -> anyhow::Result<Self> {
                Ok(Self { #(#from_pg_fields),* })
            }
        }

        impl ::rquery_orm::mapping::FromRowWithPrefix for #struct_name {
            fn from_row_ms_with(row: &tiberius::Row, prefix: &str) -> anyhow::Result<Self> {
                Ok(Self { #(#from_ms_fields_with_prefix),* })
            }
            fn from_row_pg_with(row: &tokio_postgres::Row, prefix: &str) -> anyhow::Result<Self> {
                Ok(Self { #(#from_pg_fields_with_prefix),* })
            }
        }

        impl ::rquery_orm::mapping::Validatable for #struct_name {
            fn validate(&self) -> Result<(), Vec<String>> {
                let mut errors = Vec::new();
                #(#validate_stmts)*
                if errors.is_empty() { Ok(()) } else { Err(errors) }
            }
        }

        impl ::rquery_orm::mapping::Persistable for #struct_name {
            fn build_insert(&self, style: ::rquery_orm::query::PlaceholderStyle) -> (String, Vec<::rquery_orm::query::SqlParam>, bool) {
                use ::rquery_orm::query::ToParam;
                let mut cols = Vec::new();
                let mut vals = Vec::new();
                let mut params = Vec::new();
                let mut idx = 1;
                #(#insert_stmts)*
                let sql = format!("INSERT INTO {} ({}) VALUES ({})", #table_name, cols.join(", "), vals.join(", "));
                (sql, params, #has_identity)
            }

            fn build_update(&self, style: ::rquery_orm::query::PlaceholderStyle) -> (String, Vec<::rquery_orm::query::SqlParam>) {
                use ::rquery_orm::query::ToParam;
                let mut sets = Vec::new();
                let mut wheres = Vec::new();
                let mut params = Vec::new();
                let mut idx = 1;
                #(#update_set_stmts)*
                #(#update_where_stmts)*
                let sql = format!("UPDATE {} SET {} WHERE {}", #table_name, sets.join(", "), wheres.join(" AND "));
                (sql, params)
            }

            fn build_delete(&self, style: ::rquery_orm::query::PlaceholderStyle) -> (String, Vec<::rquery_orm::query::SqlParam>) {
                use ::rquery_orm::query::ToParam;
                let mut wheres = Vec::new();
                let mut params = Vec::new();
                let mut idx = 1;
                #(#delete_where_stmts)*
                let sql = format!("DELETE FROM {} WHERE {}", #table_name, wheres.join(" AND "));
                (sql, params)
            }

            fn build_delete_by_key(key: ::rquery_orm::query::SqlParam, style: ::rquery_orm::query::PlaceholderStyle) -> (String, Vec<::rquery_orm::query::SqlParam>) {
                let placeholder = match style {
                    ::rquery_orm::query::PlaceholderStyle::AtP => "@P1".to_string(),
                    ::rquery_orm::query::PlaceholderStyle::Dollar => "$1".to_string(),
                };
                let sql = format!("DELETE FROM {} WHERE {} = {}", #table_name, #first_key_col_literal, placeholder);
                (sql, vec![key])
            }
        }

        impl #struct_name {
            pub const TABLE: &'static str = #table_name_lit;
            #(#assoc_consts)*
        }

        #(#key_trait_impls)*
    };

    TokenStream::from(expanded)
}
