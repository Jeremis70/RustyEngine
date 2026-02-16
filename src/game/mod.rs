pub mod demos;

const ACTIVE_DEMO: DemoSelector = DemoSelector::Grass;

#[derive(Clone, Copy)]
enum DemoSelector {
    ActionsDemo,
    Showcase,
    Template,
    Sprite,
    Font,
    AssetCache,
    DoomLike,
    Grass,
}

pub fn install_active_demo(engine: &mut crate::core::engine::Engine) {
    match ACTIVE_DEMO {
        DemoSelector::ActionsDemo => demos::actions_demo::install(engine),
        DemoSelector::Showcase => demos::showcase::install(engine),
        DemoSelector::Template => demos::template::install(engine),
        DemoSelector::Sprite => demos::sprite::install(engine),
        DemoSelector::Font => demos::font::install(engine),
        DemoSelector::AssetCache => demos::asset_cache::install(engine),
        DemoSelector::DoomLike => demos::doom_like::install(engine),
        DemoSelector::Grass => demos::grass::install(engine),
    }
}

pub fn list_available() -> [&'static str; 8] {
    [
        "ActionsDemo",
        "Showcase",
        "Template",
        "Font",
        "Sprite",
        "AssetCache",
        "DoomLike",
        "Grass",
    ]
}
