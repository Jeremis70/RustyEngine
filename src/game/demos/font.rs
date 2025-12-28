use log::info;

use crate::core::engine::Engine;
use crate::core::engine_state::EngineState;
use crate::graphics::Text;
use crate::math::{Color, Vec2};
use crate::render::{Drawable, Rectangle, context::RenderContext};

/// Font rendering demo showing the layout() + draw() pattern
pub fn install(engine: &mut Engine) {
    info!("Font demo loaded.");

    // Load a font at a reference size (e.g., 48px for high quality)
    // Then we can scale to any size in Text instances!
    let font_id = engine
        .assets
        .load_font_latin1("src\\game\\assets\\Minecraft.ttf", 48.0)
        .expect("Failed to load font");
    info!("Font loaded with ID: {:?}", font_id);

    // Create text instances at different sizes from the same font atlas
    let mut text1 = Text::new(font_id, "Large Text (48px)", 48, Color::WHITE);
    text1.transform.position = Vec2::new(50.0, 50.0);
    text1.transform.scale = Vec2::new(1.15, 1.15);
    text1.transform.rotation = 0.25;
    text1.layout(&engine.assets);
    info!(
        "text1 topleft={:?} size={:?}",
        text1.transform.position,
        text1.size()
    );

    let mut text2 = Text::new(font_id, "Medium Text (32px)", 32, Color::rgb(100, 200, 255));
    text2.transform.position = Vec2::new(50.0, 120.0);
    // Test: rotate around center but keep top-left anchored.
    text2.transform.origin = Vec2::new(0.5, 0.5);
    text2.layout(&engine.assets);
    text2.set_topleft(Vec2::new(50.0, 120.0));
    text2.transform.rotation = -0.20;

    let mut text3 = Text::new(font_id, "Small Text (16px)", 16, Color::rgb(255, 200, 100));
    text3.transform.position = Vec2::new(50.0, 170.0);
    text3.layout(&engine.assets);

    // Using the with_spacing constructor
    let mut text4 = Text::with_spacing(
        font_id,
        "Line spacing\nand letter spacing\nwork too!",
        24,
        Color::rgb(150, 255, 150),
        1.5, // line_height
        2.0, // letter_spacing
    );
    text4.transform.position = Vec2::new(50.0, 210.0);
    text4.layout(&engine.assets);

    // === TEST SPECIAL CHARACTERS ===
    let mut test_numbers = Text::new(font_id, "Numbers: 0123456789", 20, Color::WHITE);
    test_numbers.transform.position = Vec2::new(400.0, 50.0);
    test_numbers.layout(&engine.assets);

    let mut test_punctuation = Text::new(
        font_id,
        "Punctuation: !\"#$%&'()*+,-./:;<=>?@",
        16,
        Color::rgb(200, 200, 200),
    );
    test_punctuation.transform.position = Vec2::new(400.0, 80.0);
    test_punctuation.layout(&engine.assets);

    let mut test_brackets = Text::new(
        font_id,
        "Brackets: []{}()<>|\\",
        16,
        Color::rgb(200, 200, 200),
    );
    test_brackets.transform.position = Vec2::new(400.0, 105.0);
    test_brackets.layout(&engine.assets);

    let mut test_symbols = Text::new(font_id, "Symbols: ^_`~", 16, Color::rgb(200, 200, 200));
    test_symbols.transform.position = Vec2::new(400.0, 130.0);
    test_symbols.layout(&engine.assets);

    // Test missing characters (accents not in ASCII 32-126)
    let mut test_accents = Text::new(
        font_id,
        "Accents (may not render): éàèùç",
        16,
        Color::rgb(255, 100, 100),
    );
    test_accents.transform.position = Vec2::new(400.0, 160.0);
    test_accents.layout(&engine.assets);

    // Test empty glyphs (spaces)
    let mut test_spaces = Text::new(font_id, "M u l t i  S p a c e s", 16, Color::WHITE);
    test_spaces.transform.position = Vec2::new(400.0, 190.0);
    test_spaces.layout(&engine.assets);

    // Test very long line
    let mut test_long = Text::new(
        font_id,
        "Very long line to test horizontal overflow behavior",
        14,
        Color::rgb(255, 200, 100),
    );
    test_long.transform.position = Vec2::new(400.0, 220.0);
    test_long.layout(&engine.assets);

    // Test multiple consecutive newlines
    let mut test_newlines = Text::new(font_id, "Line 1\n\n\nLine 2", 16, Color::rgb(150, 255, 200));
    test_newlines.transform.position = Vec2::new(400.0, 250.0);
    test_newlines.layout(&engine.assets);

    // Test edge case: empty string
    let mut test_empty = Text::new(font_id, "", 16, Color::WHITE);
    test_empty.transform.position = Vec2::new(400.0, 350.0);
    test_empty.layout(&engine.assets);

    engine.events.on_update(|_state: &EngineState| {
        // Game logic updates here
    });

    engine.events.on_render(move |ctx: &mut RenderContext| {
        ctx.clear(Color::from((30, 30, 40)));

        // --- Visual tests: bounding boxes ---
        // Goal: verify that `transform.position` is the text's tight top-left and that `size()` is consistent.
        let mut draw_bounds = |t: &Text, color: Color| {
            let mut r = Rectangle::new_outline(Vec2::ZERO, t.size(), color, 1.0);
            r.transform = t.transform.clone();
            r.draw(ctx);
        };

        draw_bounds(&text1, Color::rgb(255, 255, 255));
        draw_bounds(&text2, Color::rgb(100, 200, 255));
        draw_bounds(&text3, Color::rgb(255, 200, 100));
        draw_bounds(&text4, Color::rgb(150, 255, 150));
        draw_bounds(&test_numbers, Color::rgb(255, 255, 255));
        draw_bounds(&test_punctuation, Color::rgb(200, 200, 200));
        draw_bounds(&test_brackets, Color::rgb(200, 200, 200));
        draw_bounds(&test_symbols, Color::rgb(200, 200, 200));
        draw_bounds(&test_accents, Color::rgb(255, 100, 100));
        draw_bounds(&test_spaces, Color::rgb(255, 255, 255));
        draw_bounds(&test_long, Color::rgb(255, 200, 100));
        draw_bounds(&test_newlines, Color::rgb(150, 255, 200));
        draw_bounds(&test_empty, Color::rgb(255, 255, 255));

        // Draw texts at different sizes from the same font atlas!
        text1.draw(ctx);
        text2.draw(ctx);
        text3.draw(ctx);
        text4.draw(ctx);

        // Draw special character tests
        test_numbers.draw(ctx);
        test_punctuation.draw(ctx);
        test_brackets.draw(ctx);
        test_symbols.draw(ctx);
        test_accents.draw(ctx);
        test_spaces.draw(ctx);
        test_long.draw(ctx);
        test_newlines.draw(ctx);
        test_empty.draw(ctx);
    });

    engine.events.on_mouse_move(|_pos| {
        // Mouse interaction handling
    });
}
