use stayhydated_xtask::web::WebBuildConfig;

pub(crate) fn run() -> anyhow::Result<()> {
    let root = stayhydated_xtask::workspace_root_from_xtask_manifest()?;
    stayhydated_xtask::web::build(
        WebBuildConfig::github_pages(&root)
            .package("web")
            .route_manifest(web::route_manifest())
            .build(),
    )
}
