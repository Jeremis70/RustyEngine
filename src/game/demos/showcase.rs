use std::cell::RefCell;
use std::f32::consts::FRAC_PI_4;
use std::rc::Rc;

use log::{info, warn};

use crate::audio::{AudioResult, AudioSystem};
use crate::core::color::Color;
use crate::core::engine::Engine;
use crate::core::engine_state::EngineState;
use crate::core::events::Position;
use crate::core::render_context::RenderContext;
use crate::math::vec2::Vec2;
use crate::render::shapes::Polygon;
use crate::render::{
    Circle, Collider, Drawable, Ellipse, Line, Polyline, Rectangle, Transform2d, Triangle,
};

const MUSIC_TRACK: &str =
    r#"D:\Code\Rust\RustyEngine\src\game\audio\toby fox - UNDERTALE Soundtrack - 17 Snowy.flac"#;

/// Installs the animated showcase demo featuring shapes, collisions and audio.
pub fn install(engine: &mut Engine) {
    let scene = Rc::new(RefCell::new(Scene::new()));

    {
        let scene = Rc::clone(&scene);
        engine.events.on_update(move |state: &EngineState| {
            let dt = state.delta_seconds();
            scene.borrow_mut().update(dt);
        });
    }

    {
        let scene = Rc::clone(&scene);
        engine.events.on_render(move |ctx: &mut RenderContext| {
            ctx.clear(Color::BLUE);
            scene.borrow().render(ctx);
        });
    }

    {
        let scene = Rc::clone(&scene);
        engine.events.on_mouse_move(move |pos: &Position| {
            scene.borrow_mut().handle_pointer(Vec2::from(pos));
        });
    }

    if let Err(err) = install_audio(&mut engine.audio) {
        warn!("Impossible de lancer la musique: {err}");
    }
}

fn install_audio(audio: &mut AudioSystem) -> AudioResult<()> {
    let path = asset_path(MUSIC_TRACK);
    if !path.exists() {
        warn!("Fichier audio introuvable: {}", path.display());
        return Ok(());
    }

    let sound = audio.load(&path)?;
    audio.play(sound)
}

fn asset_path(relative: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join(relative)
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum ShapeId {
    Rectangle,
    Triangle,
    Circle,
    Polygon,
    Ellipse,
    Line,
    Polyline,
}

impl ShapeId {
    fn label(self) -> &'static str {
        match self {
            ShapeId::Rectangle => "le rectangle",
            ShapeId::Triangle => "le triangle",
            ShapeId::Circle => "le cercle",
            ShapeId::Polygon => "le polygone",
            ShapeId::Ellipse => "l'ellipse",
            ShapeId::Line => "la ligne",
            ShapeId::Polyline => "la polyligne",
        }
    }
}

struct Scene {
    rectangle: Rectangle,
    triangle: Triangle,
    circle: Circle,
    polygon: Polygon,
    ellipse: Ellipse,
    line: Line,
    polyline: Polyline,
    hovered: Vec<ShapeId>,
    time: f32,
}

impl Scene {
    fn new() -> Self {
        let mut rectangle = Rectangle::new(Vec2::new(0.0, 0.0), Vec2::new(100.0, 50.0), Color::RED);
        rectangle.set_origin_center_keep_position();
        rectangle.translate(Vec2::new(200.0, 120.0));
        rectangle.scale_uniform(1.35);
        rectangle.rotate(FRAC_PI_4);

        let mut triangle = Triangle::new(
            Vec2::new(200.0, 200.0),
            Vec2::new(981.0, 345.0),
            Vec2::new(832.0, 125.0),
            Color::GREEN,
        );
        triangle.set_origin_center_keep_position();
        triangle.translate(Vec2::new(-120.0, -80.0));
        triangle.rotate(-FRAC_PI_4 * 0.5);

        let mut circle = Circle::new(Vec2::new(400.0, 200.0), 75.0, Color::YELLOW);
        circle.set_origin_center_keep_position();
        circle.scale_uniform(1.1);
        circle.rotate(FRAC_PI_4 * 0.5);

        let mut polygon = Polygon::new(
            vec![
                Vec2::new(520.0, 60.0),
                Vec2::new(640.0, 140.0),
                Vec2::new(600.0, 210.0),
                Vec2::new(690.0, 260.0),
                Vec2::new(510.0, 320.0),
                Vec2::new(470.0, 170.0),
            ],
            Color::new(0.9, 0.3, 0.8, 1.0),
        );
        polygon.set_origin_center_keep_position();
        polygon.translate(Vec2::new(40.0, 20.0));
        polygon.set_scale(Vec2::new(0.85, 1.15));
        polygon.rotate(FRAC_PI_4 * 0.25);

        let mut ellipse = Ellipse::new(
            Vec2::new(600.0, 260.0),
            60.0,
            40.0,
            Color::new(0.2, 0.9, 1.0, 1.0),
        );
        ellipse.set_origin_center_keep_position();
        ellipse.set_scale(Vec2::new(1.1, 0.8));
        ellipse.rotate(-FRAC_PI_4 * 0.4);

        let mut line = Line::new(
            Vec2::new(80.0, 340.0),
            Vec2::new(280.0, 290.0),
            Color::WHITE,
            10.0,
        );
        line.set_origin_center_keep_position();
        line.scale_uniform(1.15);
        line.rotate(FRAC_PI_4 * 0.15);

        let mut polyline = Polyline::new(
            vec![
                Vec2::new(320.0, 340.0),
                Vec2::new(420.0, 360.0),
                Vec2::new(480.0, 320.0),
                Vec2::new(540.0, 360.0),
                Vec2::new(620.0, 330.0),
            ],
            Color::WHITE,
            8.0,
        );
        polyline.set_origin_center_keep_position();
        polyline.translate(Vec2::new(-40.0, -60.0));
        polyline.rotate(-FRAC_PI_4 * 0.2);

        let scene = Self {
            rectangle,
            triangle,
            circle,
            polygon,
            ellipse,
            line,
            polyline,
            hovered: Vec::new(),
            time: 0.0,
        };
        scene.log_intersections();
        scene
    }

    fn update(&mut self, dt: f32) {
        self.time += dt;

        self.rectangle.rotate(0.35 * dt);
        self.triangle.rotate(-0.2 * dt);
        self.polygon.rotate(0.25 * dt);
        self.ellipse.rotate(-0.4 * dt);
        self.line.rotate(0.15 * dt);
        self.polyline.rotate(0.2 * dt);

        let pulsate = 1.0 + 0.05 * (self.time * 2.0).sin();
        self.circle.set_scale(Vec2::new(pulsate, pulsate));
    }

    fn render(&self, ctx: &mut RenderContext) {
        self.triangle.draw(ctx);
        self.circle.draw(ctx);
        self.polygon.draw(ctx);
        self.ellipse.draw(ctx);
        self.line.draw(ctx);
        self.polyline.draw(ctx);
        self.rectangle.draw(ctx);
    }

    fn handle_pointer(&mut self, cursor: Vec2) {
        let mut hits = Vec::new();
        if self.rectangle.contains_point(cursor) {
            hits.push(ShapeId::Rectangle);
        }
        if self.triangle.contains_point(cursor) {
            hits.push(ShapeId::Triangle);
        }
        if self.circle.contains_point(cursor) {
            hits.push(ShapeId::Circle);
        }
        if self.polygon.contains_point(cursor) {
            hits.push(ShapeId::Polygon);
        }
        if self.ellipse.contains_point(cursor) {
            hits.push(ShapeId::Ellipse);
        }
        if self.line.contains_point(cursor) {
            hits.push(ShapeId::Line);
        }
        if self.polyline.contains_point(cursor) {
            hits.push(ShapeId::Polyline);
        }
        hits.sort();

        if hits != self.hovered {
            for id in hits.iter().filter(|id| !self.hovered.contains(id)) {
                info!("La souris est entrée dans {}", id.label());
            }
            for id in self.hovered.iter().filter(|id| !hits.contains(id)) {
                info!("La souris a quitté {}", id.label());
            }
            self.hovered = hits;
        }
    }

    fn log_intersections(&self) {
        self.log_pair(
            ShapeId::Rectangle,
            &self.rectangle,
            ShapeId::Triangle,
            &self.triangle,
        );
        self.log_pair(
            ShapeId::Rectangle,
            &self.rectangle,
            ShapeId::Circle,
            &self.circle,
        );
        self.log_pair(
            ShapeId::Triangle,
            &self.triangle,
            ShapeId::Polygon,
            &self.polygon,
        );
        self.log_pair(
            ShapeId::Ellipse,
            &self.ellipse,
            ShapeId::Polyline,
            &self.polyline,
        );
        self.log_pair(
            ShapeId::Line,
            &self.line,
            ShapeId::Rectangle,
            &self.rectangle,
        );
    }

    fn log_pair(&self, a_id: ShapeId, a: &dyn Collider, b_id: ShapeId, b: &dyn Collider) {
        let result = if a.intersects(b) { "oui" } else { "non" };
        info!("{} intersecte {} ? {}", a_id.label(), b_id.label(), result);
    }
}
