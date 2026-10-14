mod components;
mod examples;
mod i18n;
mod site;

pub use site::App;

pub fn route_manifest() -> stayhydated_site::SiteRouteManifest {
    site::site().route_manifest::<site::Route>()
}

#[cfg(test)]
mod tests {
    #[test]
    fn dioxus_assets_are_registered_from_the_library_target() {
        let modules = es_fluent_manager_dioxus::DioxusI18nAssetModules::discovered();
        assert_eq!(modules.as_slice().len(), 1);
    }

    #[test]
    fn pages_manifest_includes_every_example_and_promised_static_output() {
        let manifest = super::route_manifest();
        let paths = manifest
            .application_paths()
            .iter()
            .map(|p| p.as_str())
            .collect::<Vec<_>>();
        assert_eq!(paths.len(), 12);
        for page in <super::site::Page as iso_types::VariantArray>::VARIANTS {
            assert!(
                paths.contains(&page.route().to_string().as_str()),
                "{page:?}"
            );
        }
        for path in ["/book/", "/llms.txt", "/llms-full.txt"] {
            assert!(manifest.static_paths().iter().any(|p| p.as_str() == path));
        }
    }
}
