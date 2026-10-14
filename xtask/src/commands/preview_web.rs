use stayhydated_xtask::preview::StaticSitePreviewConfig;

pub(crate) fn run() -> anyhow::Result<()> {
    let root = stayhydated_xtask::workspace_root_from_xtask_manifest()?;
    stayhydated_xtask::preview::serve(
        &StaticSitePreviewConfig::builder()
            .workspace_root(root)
            .dist_dir("web/dist")
            .base_path("iso-types")
            .build_hint("Run `just web-build` first.")
            .build(),
    )
}
