use std::mem::take;

use base64::{engine::general_purpose::STANDARD as engine, Engine};
use macro_railroad::{diagram, lowering, parser};
use proc_macro2::Span;
use quote::ToTokens;
use rand::distributions::{Alphanumeric, DistString};
use syn::{parse_quote, spanned::Spanned, AttrStyle, LitStr, Meta};

/// Annotate a `macro_rules!` statement with this macro to include a diagram of the
/// inputs in the documentation.
///
/// # Usage
/// ## Without arguments
/// `#[macro_railroad_annotation::generate_railroad]` tries to position the image in the documentation
/// exactly where the macro was invoked (so just write it in the middle of your doc comments).
///
/// The caveat is that this only works on nightly compilers (which includes docs.rs), and on stable
/// compilers it will fall back to placing the image way at the top
///
/// ## With arguments
/// `#[macro_railroad_annotation::generate_railroad("some_label")]` doesn't insert the image at all, but generates
/// a label you can refer to with `![][some_label]` in the markdown.
///
/// More information in the readme.
#[manyhow::manyhow(proc_macro_attribute)]
pub fn generate_railroad(
    attr: proc_macro2::TokenStream,
    mut item: syn::ItemMacro,
) -> manyhow::Result<syn::ItemMacro> {
    let macro_span = attr.span().byte_range().end;
    let attr: Option<syn::LitStr> = syn::parse2(attr)?;
    let mut attrs = take(&mut item.attrs);
    // Position in the attributes where the macro was invoked
    let macro_pos = attrs
        .iter()
        .enumerate()
        .find_map(|(i, attr)| (attr.span().byte_range().start >= macro_span).then_some(i))
        .unwrap_or_default();

    let macro_input = item.to_token_stream().to_string();

    let macro_rules = parser::parse(&macro_input)?;
    let mut tree = lowering::MacroRules::from(macro_rules);
    tree.remove_internal();
    tree.foldcommontails();
    tree.normalize();
    let mut diagram = diagram::into_diagram(tree, true);
    diagram.add_default_css();
    diagram::add_default_css(&mut diagram);

    let svg_string = diagram.to_string();

    let encoded = engine.encode(svg_string);

    let given_label = attr.is_some();
    let label = attr
        .map(|lit| lit.value())
        .unwrap_or_else(|| Alphanumeric.sample_string(&mut rand::thread_rng(), 16));

    let doc_string = LitStr::new(
        &format!("\n \n  [{label}]: data:image/svg+xml;base64,{encoded}"),
        Span::mixed_site(),
    );
    let doc_attr = parse_quote! {
        #[doc = #doc_string]
    };

    let pos = attrs
        .iter()
        .enumerate()
        .rev()
        .find_map(|(i, p)| {
            (p.style == AttrStyle::Outer
                && matches!(p.meta, Meta::NameValue(ref m) if m.path.is_ident("doc")))
            .then_some(i + 1)
        })
        .unwrap_or(attrs.len());

    attrs.insert(pos, doc_attr);
    if !given_label {
        let alt_text = item.ident.as_ref().map_or_else(
            || String::from("below"),
            |i| {
                let ident_str = i.to_string();
                format!("[`{ident_str}`]")
            },
        );
        let alt_text_placeholders =
            "=".repeat(alt_text.len() - if item.ident.is_some() { 4 } else { 0 });
        let image_ref = LitStr::new(
            &format!(" ![=============================================={alt_text_placeholders}\n_Here would be a railroad diagram of the macro {alt_text}_\n=============================================={alt_text_placeholders}][{label}]\n\n"),
            Span::mixed_site(),
        );
        attrs.insert(
            macro_pos,
            parse_quote! {
                #[doc = #image_ref]
            },
        )
    }
    item.attrs = attrs;

    Ok(item)
}
