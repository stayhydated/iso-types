//! ISO subset expansion: parse and validate first, then emit Rust.

use proc_macro2::TokenStream;

mod data;
mod emit;
mod model;

pub(crate) fn expand(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let model = model::Model::parse(attr, item)?;
    Ok(emit::emit(&model))
}

#[cfg(test)]
mod tests {
    use super::expand;
    use quote::quote;
    #[test]
    fn invalid_declarations_report_the_input_error() {
        let cases = [
            (
                quote!(standard = "3166-2", country = "ZZ"),
                quote!(
                    enum Codes {}
                ),
                "unavailable country filter",
            ),
            (
                quote!(standard = "3166-1", country = "US"),
                quote!(
                    enum Codes {}
                ),
                "unavailable country filter",
            ),
            (
                quote!(standard = "units", category = "Length"),
                quote!(
                    enum Codes {}
                ),
                "unavailable category filter",
            ),
            (
                quote!(standard = "units", category = "mass", codes = ["meter"]),
                quote!(
                    enum Codes {}
                ),
                "select no codes",
            ),
            (
                quote!(standard = "3166-1"),
                quote!(
                    #[strum(prefix = "market-")]
                    enum Codes {}
                ),
                "own canonical codes",
            ),
            (
                quote!(standard = "3166-1"),
                quote!(
                    #[strum(suffix = "-market")]
                    enum Codes {}
                ),
                "own canonical codes",
            ),
            (
                quote!(),
                quote!(
                    enum Codes {}
                ),
                "standard is required",
            ),
            (
                quote!(standard = "9999"),
                quote!(
                    enum Codes {}
                ),
                "unsupported catalog",
            ),
            (
                quote!(standard = "3166-1", codes = ["ZZ"]),
                quote!(
                    enum Codes {}
                ),
                "canonical code",
            ),
            (
                quote!(standard = "639-3", codes = ["aaa"]),
                quote!(
                    enum Codes {}
                ),
                "canonical code",
            ),
            (
                quote!(standard = "3166-3"),
                quote!(
                    enum Codes {}
                ),
                "unsupported catalog",
            ),
            (
                quote!(standard = "639-5"),
                quote!(
                    enum Codes {}
                ),
                "unsupported catalog",
            ),
            (
                quote!(standard = "639-3", codes = ["ang"]),
                quote!(
                    enum Codes {}
                ),
                "canonical code",
            ),
            (
                quote!(standard = "15924", codes = ["Egyp"]),
                quote!(
                    enum Codes {}
                ),
                "canonical code",
            ),
            (
                quote!(standard = "3166-1", codes = ["US", "US"]),
                quote!(
                    enum Codes {}
                ),
                "duplicate catalog code",
            ),
            (
                quote!(standard = "3166-1", codes = []),
                quote!(
                    enum Codes {}
                ),
                "select at least one code",
            ),
            (
                quote!(standard = "3166-1"),
                quote!(
                    enum Codes {
                        A,
                    }
                ),
                "empty enum",
            ),
            (
                quote!(standard = "3166-1"),
                quote!(
                    enum Codes<T> {}
                ),
                "without generics",
            ),
            (
                quote!(standard = "3166-1", standard = "639-1"),
                quote!(
                    enum Codes {}
                ),
                "once each",
            ),
        ];
        for (attributes, item, message) in cases {
            let error = expand(attributes, item).unwrap_err();
            assert!(error.to_string().contains(message), "{error}");
        }
    }
}
