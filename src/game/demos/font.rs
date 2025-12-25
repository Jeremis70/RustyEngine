use log::info;

use crate::core::engine::Engine;
use crate::core::engine_state::EngineState;
use crate::math::Color;
use crate::render::context::RenderContext;

/// Minimal scaffold that illustrates where to plug update and render code.
pub fn install(engine: &mut Engine) {
    info!("Template demo chargée: ajoutez votre logique ici.");

    engine.events.on_update(|_state: &EngineState| {
        // Ajoutez ici votre logique de jeu (physique, IA, etc.).
        // Utilisez `_state.delta_seconds()` pour la durée de la frame.
    });

    engine.events.on_render(|ctx: &mut RenderContext| {
        // Changez la couleur de fond selon vos besoins.
        ctx.clear(Color::BLACK);
        // Dessinez vos entités ici en utilisant ctx et les formes du moteur.
    });

    engine.events.on_mouse_move(|_pos| {
        // Captez les interactions souris/clavier ou gamepad selon votre scène.
    });
}
