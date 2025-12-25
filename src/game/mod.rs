pub mod demos;

/// Changez cette constante pour lancer une démo différente.
const ACTIVE_DEMO: DemoSelector = DemoSelector::Sprite;

#[derive(Clone, Copy)]
enum DemoSelector {
    Showcase,
    Template,
    Sprite,
    Font,
}

pub fn install_active_demo(engine: &mut crate::core::engine::Engine) {
    match ACTIVE_DEMO {
        DemoSelector::Showcase => demos::showcase::install(engine),
        DemoSelector::Template => demos::template::install(engine),
        DemoSelector::Sprite => demos::sprite::install(engine),
        DemoSelector::Font => demos::font::install(engine),
    }
}

pub fn list_available() -> [&'static str; 4] {
    ["Showcase", "Template", "Font", "Sprite"]
}
