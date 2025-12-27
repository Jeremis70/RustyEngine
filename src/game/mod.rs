pub mod demos;

const ACTIVE_DEMO: DemoSelector = DemoSelector::Sprite;

#[derive(Clone, Copy)]
enum DemoSelector {
    Showcase,
    Template,
    Sprite,
    Font,
    AssetCache,
}

pub fn install_active_demo(engine: &mut crate::core::engine::Engine) {
    match ACTIVE_DEMO {
        DemoSelector::Showcase => demos::showcase::install(engine),
        DemoSelector::Template => demos::template::install(engine),
        DemoSelector::Sprite => demos::sprite::install(engine),
        DemoSelector::Font => demos::font::install(engine),
        DemoSelector::AssetCache => demos::asset_cache::install(engine),
    }
}

pub fn list_available() -> [&'static str; 5] {
    ["Showcase", "Template", "Font", "Sprite", "AssetCache"]
}
