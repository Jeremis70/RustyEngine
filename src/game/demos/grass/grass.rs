use std::collections::HashMap;

use crate::core::assets::{ImageAsset, ImageId};
use crate::math::vec2::Vec2;

#[derive(Clone, Debug)]
pub struct GrassAssets {
    pub blades: Vec<ImageAsset>,
}

impl GrassAssets {
    pub fn new(mut blades: Vec<ImageAsset>) -> Self {
        for blade in &mut blades {
            apply_black_colorkey_in_place(blade);
        }
        Self { blades }
    }

    fn render_blade_into(
        &self,
        target: &mut ImageAsset,
        blade_id: usize,
        center: (f32, f32),
        rotation_degrees: f32,
        shade_amount: u8,
    ) {
        let blade = &self.blades[blade_id];
        let rotated = rotate_rgba_bilinear(blade, rotation_degrees);

        let shade_factor = 1.0 - (shade_amount as f32 / 255.0) * (rotation_degrees.abs() / 90.0);
        let shaded = if shade_factor >= 0.999 {
            rotated
        } else {
            apply_shade(rotated, shade_factor)
        };

        let x = (center.0 - shaded.width as f32 / 2.0).round() as i32;
        let y = (center.1 - shaded.height as f32 / 2.0).round() as i32;
        blend_over(target, &shaded, x, y);
    }
}

#[derive(Clone, Debug)]
struct Blade {
    pos: (f32, f32),
    blade_id: usize,
    rotation: f32,
}

#[derive(Clone, Debug)]
pub struct GroundShadow {
    pub radius: u32,
    pub color: (u8, u8, u8),
    pub strength: u8,
    pub shift: (i32, i32),
}

impl GroundShadow {
    pub fn disabled() -> Self {
        Self {
            radius: 0,
            color: (0, 0, 1),
            strength: 0,
            shift: (0, 0),
        }
    }

    pub fn enabled(
        shadow_strength: u8,
        shadow_radius: u32,
        mut shadow_color: (u8, u8, u8),
        shadow_shift: (i32, i32),
    ) -> Self {
        if shadow_color == (0, 0, 0) {
            shadow_color = (0, 0, 1);
        }
        Self {
            radius: shadow_radius,
            color: shadow_color,
            strength: shadow_strength,
            shift: shadow_shift,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.radius != 0 && self.strength != 0
    }
}

#[derive(Clone, Debug)]
struct FormatEntry {
    count: u32,
    data: Vec<(u32, Vec<Blade>)>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct FormatId {
    amt: u32,
    config: Vec<usize>,
}

#[derive(Clone, Debug)]
pub struct GrassManager {
    assets: GrassAssets,

    // caching
    grass_id: u32,
    grass_cache: HashMap<(u32, i32), ImageId>,
    shadow_cache: HashMap<u32, ImageId>,
    formats: HashMap<FormatId, FormatEntry>,

    // tiles
    grass_tiles: HashMap<(i32, i32), GrassTile>,

    // config
    pub tile_size: i32,
    pub shade_amount: u8,
    pub stiffness: f32,
    pub max_unique: u32,
    pub vertical_place_range: (f32, f32),
    pub ground_shadow: GroundShadow,
    pub padding: i32,
}

impl GrassManager {
    pub fn new(
        grass_assets: GrassAssets,
        tile_size: i32,
        shade_amount: u8,
        stiffness: f32,
        max_unique: u32,
        vertical_place_range: (f32, f32),
        padding: i32,
    ) -> Self {
        Self {
            assets: grass_assets,
            grass_id: 0,
            grass_cache: HashMap::new(),
            shadow_cache: HashMap::new(),
            formats: HashMap::new(),
            grass_tiles: HashMap::new(),
            tile_size,
            shade_amount,
            stiffness,
            max_unique,
            vertical_place_range,
            ground_shadow: GroundShadow::disabled(),
            padding,
        }
    }

    pub fn enable_ground_shadows(
        &mut self,
        shadow_strength: u8,
        shadow_radius: u32,
        shadow_color: (u8, u8, u8),
        shadow_shift: (i32, i32),
    ) {
        if shadow_strength == 0 || shadow_radius == 0 {
            self.ground_shadow = GroundShadow::disabled();
        } else {
            self.ground_shadow =
                GroundShadow::enabled(shadow_strength, shadow_radius, shadow_color, shadow_shift);
        }
    }

    pub fn place_tile(
        &mut self,
        location: (i32, i32),
        density: u32,
        grass_options: Vec<usize>,
        rng: &mut SimpleRng,
    ) {
        if self.grass_tiles.contains_key(&location) {
            return;
        }

        let tile_location_px = (
            (location.0 * self.tile_size) as f32,
            (location.1 * self.tile_size) as f32,
        );

        let tile = GrassTile::new(
            self.tile_size,
            tile_location_px,
            density,
            grass_options,
            self,
            rng,
        );

        self.grass_tiles.insert(location, tile);
    }

    pub fn apply_force(&mut self, location_px: (f32, f32), radius: f32, dropoff: f32) {
        let location_px = (location_px.0.floor() as i32, location_px.1.floor() as i32);
        let grid_pos = (
            div_floor_i32(location_px.0, self.tile_size),
            div_floor_i32(location_px.1, self.tile_size),
        );

        let tile_range = ((radius + dropoff) / self.tile_size as f32).ceil() as i32;
        for dy in -tile_range..=tile_range {
            for dx in -tile_range..=tile_range {
                let pos = (grid_pos.0 + dx, grid_pos.1 + dy);
                if let Some(tile) = self.grass_tiles.get_mut(&pos) {
                    tile.apply_force(
                        (location_px.0 as f32, location_px.1 as f32),
                        radius,
                        dropoff,
                    );
                }
            }
        }
    }

    fn get_format(
        &mut self,
        format_id: FormatId,
        data: &[Blade],
        tile_id: u32,
        rng: &mut SimpleRng,
    ) -> Option<(u32, Vec<Blade>)> {
        match self.formats.get_mut(&format_id) {
            None => {
                self.formats.insert(
                    format_id,
                    FormatEntry {
                        count: 1,
                        data: vec![(tile_id, data.to_vec())],
                    },
                );
                None
            }
            Some(entry) if entry.count >= self.max_unique => {
                let idx = rng.gen_usize(entry.data.len());
                Some(entry.data[idx].clone())
            }
            Some(entry) => {
                entry.count += 1;
                entry.data.push((tile_id, data.to_vec()));
                None
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_render(
        &mut self,
        ctx: &mut crate::render::context::RenderContext,
        assets: &mut crate::core::assets::AssetManager,
        dt: f32,
        offset_px: (f32, f32),
        viewport_size_logical: (u32, u32),
        scale: f32,
        rot_function: Option<&dyn Fn(f32, f32) -> i32>,
    ) {
        let visible_tile_range = (
            (viewport_size_logical.0 as i32 / self.tile_size) + 1,
            (viewport_size_logical.1 as i32 / self.tile_size) + 1,
        );

        let base_pos = (
            div_floor_i32(offset_px.0.floor() as i32, self.tile_size),
            div_floor_i32(offset_px.1.floor() as i32, self.tile_size),
        );

        let mut render_list = Vec::new();
        for y in 0..visible_tile_range.1 {
            for x in 0..visible_tile_range.0 {
                let pos = (base_pos.0 + x, base_pos.1 + y);
                if self.grass_tiles.contains_key(&pos) {
                    render_list.push(pos);
                }
            }
        }

        // Split borrows so we can mutably access tiles + caches without borrowing `self` twice.
        let ground_shadow = self.ground_shadow.clone();
        let shade_amount = self.shade_amount;
        let stiffness = self.stiffness;
        let grass_assets = &self.assets;
        let tiles = &mut self.grass_tiles;
        let grass_cache = &mut self.grass_cache;
        let shadow_cache = &mut self.shadow_cache;

        // Render shadows first.
        if ground_shadow.is_enabled() {
            for pos in &render_list {
                if let Some(tile) = tiles.get_mut(pos) {
                    tile.render_shadow(ctx, assets, offset_px, shadow_cache, &ground_shadow, scale);
                }
            }
        }

        // Render grass.
        for pos in render_list {
            if let Some(tile) = tiles.get_mut(&pos) {
                tile.render(
                    ctx,
                    assets,
                    grass_assets,
                    grass_cache,
                    shadow_cache,
                    shade_amount,
                    stiffness,
                    &ground_shadow,
                    dt,
                    offset_px,
                    scale,
                );
                if let Some(rot_fn) = rot_function {
                    let r = rot_fn(tile.loc.0, tile.loc.1);
                    tile.set_rotation(r);
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct GrassTile {
    loc: (f32, f32),
    size: i32,
    blades: Vec<Blade>,

    master_rotation: i32,
    precision: i32,
    inc_degrees: f32,

    padding: i32,

    base_id: u32,
    custom_blades: Option<Vec<Blade>>,

    render_key: (u32, i32),
    true_rotation_degrees: f32,
}

impl GrassTile {
    fn new(
        tile_size: i32,
        location_px: (f32, f32),
        amt: u32,
        config: Vec<usize>,
        gm: &mut GrassManager,
        rng: &mut SimpleRng,
    ) -> Self {
        let precision = 30;
        let inc_degrees = 90.0 / precision as f32;
        let padding = gm.padding;

        let mut blades: Vec<Blade> = Vec::with_capacity(amt as usize);
        let (place_min, place_max) = gm.vertical_place_range;
        let y_range = (place_max - place_min).max(0.0);

        for _ in 0..amt {
            let blade_id = config[rng.gen_usize(config.len())];

            let y_pos = if y_range > 0.0 {
                rng.gen_f32() * y_range + place_min
            } else {
                place_min
            };

            blades.push(Blade {
                pos: (rng.gen_f32() * tile_size as f32, y_pos * tile_size as f32),
                blade_id,
                rotation: rng.gen_f32() * 30.0 - 15.0,
            });
        }

        // Layer back-to-front based on blade id (matches Python sort key lambda x: x[1]).
        blades.sort_by_key(|b| b.blade_id);

        let base_id = gm.grass_id;
        gm.grass_id += 1;

        // Deduplicate formats to limit RAM.
        let format_id = FormatId {
            amt,
            config: config.clone(),
        };

        let (base_id, blades) =
            if let Some((tile_id, data)) = gm.get_format(format_id, &blades, base_id, rng) {
                (tile_id, data)
            } else {
                (base_id, blades)
            };

        let mut out = Self {
            loc: location_px,
            size: tile_size,
            blades,
            master_rotation: 0,
            precision,
            inc_degrees,
            padding,
            base_id,
            custom_blades: None,
            render_key: (base_id, 0),
            true_rotation_degrees: 0.0,
        };

        out.update_render_data();
        out
    }

    fn update_render_data(&mut self) {
        self.render_key = (self.base_id, self.master_rotation);
        self.true_rotation_degrees = self.inc_degrees * self.master_rotation as f32;
    }

    fn set_rotation(&mut self, rotation: i32) {
        self.master_rotation = rotation;
        self.update_render_data();
    }

    fn apply_force(&mut self, force_point_px: (f32, f32), force_radius: f32, force_dropoff: f32) {
        if self.custom_blades.is_none() {
            self.custom_blades = Some(self.blades.clone());
        }

        let blades = self.custom_blades.as_mut().unwrap();
        for (i, blade) in self.blades.iter().enumerate() {
            let bx = self.loc.0 + blade.pos.0;
            let by = self.loc.1 + blade.pos.1;
            let dx = bx - force_point_px.0;
            let dy = by - force_point_px.1;
            let dis = (dx * dx + dy * dy).sqrt();

            let force = if dis < force_radius {
                2.0
            } else {
                let dis2 = (dis - force_radius).max(0.0);
                1.0 - (dis2 / force_dropoff).min(1.0)
            };

            let dir = if force_point_px.0 > bx { 1.0 } else { -1.0 };
            let desired = blade.rotation + dir * force * 90.0;

            let should_update =
                (blades[i].rotation - self.blades[i].rotation).abs() <= (force.abs() * 90.0);
            if should_update {
                blades[i].rotation = desired;
            }
        }
    }

    fn render_shadow(
        &mut self,
        ctx: &mut crate::render::context::RenderContext,
        assets: &mut crate::core::assets::AssetManager,
        offset_px: (f32, f32),
        shadow_cache: &mut HashMap<u32, ImageId>,
        ground_shadow: &GroundShadow,
        scale: f32,
    ) {
        if !ground_shadow.is_enabled() {
            return;
        }

        let image_id = match shadow_cache.get(&self.base_id).copied() {
            Some(id) => id,
            None => {
                // Generate shadow image once per base layout.
                let shadow_asset = self.render_shadow_tile_asset(ground_shadow);
                let id = assets
                    .load_image_from_asset(shadow_asset)
                    .expect("failed to load runtime shadow image asset");
                shadow_cache.insert(self.base_id, id);
                id
            }
        };

        let img = assets
            .get_image(image_id)
            .expect("shadow image should exist after load");

        let shadow_shift = ground_shadow.shift;
        let pos_x =
            (self.loc.0 - offset_px.0 - self.padding as f32 + shadow_shift.0 as f32) * scale;
        let pos_y =
            (self.loc.1 - offset_px.1 - self.padding as f32 + shadow_shift.1 as f32) * scale;

        let mut draw = crate::render::SpriteDrawData::new(image_id, img.width, img.height);
        draw.origin = Vec2::new(0.0, 0.0);
        draw.position = Vec2::new(pos_x, pos_y);
        draw.scale = Vec2::new(scale, scale);
        ctx.draw_sprite(draw);
    }

    #[allow(clippy::too_many_arguments)]
    fn render(
        &mut self,
        ctx: &mut crate::render::context::RenderContext,
        assets: &mut crate::core::assets::AssetManager,
        grass_assets: &GrassAssets,
        grass_cache: &mut HashMap<(u32, i32), ImageId>,
        shadow_cache: &mut HashMap<u32, ImageId>,
        shade_amount: u8,
        stiffness: f32,
        ground_shadow: &GroundShadow,
        dt: f32,
        offset_px: (f32, f32),
        scale: f32,
    ) {
        let image_id = if self.custom_blades.is_some() {
            // Uncached: render to a transient asset each frame.
            // We intentionally keep a stable ImageId per tile base_id to avoid unbounded growth.
            // This matches Python's behavior of rendering custom tiles uncached.
            let transient_key = (self.base_id, i32::MIN);
            let id = match grass_cache.get(&transient_key).copied() {
                Some(id) => id,
                None => {
                    let asset = self.render_tile_asset(grass_assets, shade_amount, true);
                    let id = assets
                        .load_image_from_asset(asset)
                        .expect("failed to load transient runtime tile image asset");
                    grass_cache.insert(transient_key, id);
                    id
                }
            };

            // Overwrite pixel data for transient image each frame (stable ImageId).
            let new_asset = self.render_tile_asset(grass_assets, shade_amount, true);
            assets
                .update_image_from_asset(id, new_asset)
                .expect("failed to update transient runtime tile image asset");
            id
        } else {
            // Cached path.
            if let std::collections::hash_map::Entry::Vacant(e) = grass_cache.entry(self.render_key)
            {
                let (grass_asset, shadow_asset) =
                    if ground_shadow.is_enabled() && !shadow_cache.contains_key(&self.base_id) {
                        let grass_asset = self.render_tile_asset(grass_assets, shade_amount, false);
                        let shadow_asset = self.render_shadow_tile_asset(ground_shadow);
                        (grass_asset, Some(shadow_asset))
                    } else {
                        (
                            self.render_tile_asset(grass_assets, shade_amount, false),
                            None,
                        )
                    };

                let grass_id = assets
                    .load_image_from_asset(grass_asset)
                    .expect("failed to load cached runtime tile image asset");
                e.insert(grass_id);

                if let Some(shadow_asset) = shadow_asset {
                    let shadow_id = assets
                        .load_image_from_asset(shadow_asset)
                        .expect("failed to load cached runtime shadow image asset");
                    shadow_cache.insert(self.base_id, shadow_id);
                }
            }
            grass_cache[&self.render_key]
        };

        let img = assets
            .get_image(image_id)
            .expect("tile image should exist after load");

        let pos_x = (self.loc.0 - offset_px.0 - self.padding as f32) * scale;
        let pos_y = (self.loc.1 - offset_px.1 - self.padding as f32) * scale;

        let mut draw = crate::render::SpriteDrawData::new(image_id, img.width, img.height);
        draw.origin = Vec2::new(0.0, 0.0);
        draw.position = Vec2::new(pos_x, pos_y);
        draw.scale = Vec2::new(scale, scale);
        ctx.draw_sprite(draw);

        // Relax custom blades back to base.
        if let Some(custom) = self.custom_blades.as_mut() {
            let mut matching = true;
            for (i, blade) in custom.iter_mut().enumerate() {
                blade.rotation =
                    normalize_f32(blade.rotation, stiffness * dt, self.blades[i].rotation);
                if blade.rotation != self.blades[i].rotation {
                    matching = false;
                }
            }
            if matching {
                self.custom_blades = None;
            }
        }
    }

    fn render_shadow_tile_asset(&self, ground_shadow: &GroundShadow) -> ImageAsset {
        let size = (self.size + self.padding * 2) as u32;
        let mut img = ImageAsset {
            width: size,
            height: size,
            data: vec![0u8; (size * size * 4) as usize],
        };

        let radius = ground_shadow.radius as i32;
        let (cr, cg, cb) = ground_shadow.color;
        let alpha = ground_shadow.strength;

        for blade in &self.blades {
            let cx = blade.pos.0 + self.padding as f32;
            let cy = blade.pos.1 + self.padding as f32;
            draw_filled_circle(
                &mut img,
                cx.round() as i32,
                cy.round() as i32,
                radius,
                (cr, cg, cb, alpha),
            );
        }

        img
    }

    fn render_tile_asset(
        &self,
        grass_assets: &GrassAssets,
        shade_amount: u8,
        use_custom: bool,
    ) -> ImageAsset {
        let size = (self.size + self.padding * 2) as u32;
        let mut surf = ImageAsset {
            width: size,
            height: size,
            data: vec![0u8; (size * size * 4) as usize],
        };

        let blades: &[Blade] = if use_custom {
            self.custom_blades.as_deref().unwrap_or(&self.blades)
        } else {
            &self.blades
        };

        for blade in blades {
            let rot = (blade.rotation + self.true_rotation_degrees).clamp(-90.0, 90.0);
            grass_assets.render_blade_into(
                &mut surf,
                blade.blade_id,
                (
                    blade.pos.0 + self.padding as f32,
                    blade.pos.1 + self.padding as f32,
                ),
                rot,
                shade_amount,
            );
        }

        surf
    }
}

fn div_floor_i32(a: i32, b: i32) -> i32 {
    // b assumed positive.
    let mut q = a / b;
    let r = a % b;
    if r < 0 {
        q -= 1;
    }
    q
}

fn normalize_f32(val: f32, amt: f32, target: f32) -> f32 {
    if val > target + amt {
        val - amt
    } else if val < target - amt {
        val + amt
    } else {
        target
    }
}

fn apply_black_colorkey_in_place(img: &mut ImageAsset) {
    if img.data.len() != (img.width * img.height * 4) as usize {
        return;
    }

    for px in img.data.chunks_exact_mut(4) {
        let (r, g, b) = (px[0], px[1], px[2]);
        if r == 0 && g == 0 && b == 0 {
            px[3] = 0;
        }
    }
}

fn apply_shade(mut img: ImageAsset, factor: f32) -> ImageAsset {
    let f = factor.clamp(0.0, 1.0);
    for px in img.data.chunks_exact_mut(4) {
        px[0] = ((px[0] as f32) * f).round() as u8;
        px[1] = ((px[1] as f32) * f).round() as u8;
        px[2] = ((px[2] as f32) * f).round() as u8;
    }
    img
}

fn blend_over(dst: &mut ImageAsset, src: &ImageAsset, dst_x: i32, dst_y: i32) {
    let dw = dst.width as i32;
    let dh = dst.height as i32;
    let sw = src.width as i32;
    let sh = src.height as i32;

    for sy in 0..sh {
        let dy = dst_y + sy;
        if dy < 0 || dy >= dh {
            continue;
        }
        for sx in 0..sw {
            let dx = dst_x + sx;
            if dx < 0 || dx >= dw {
                continue;
            }

            let sidx = ((sy * sw + sx) * 4) as usize;
            let didx = ((dy * dw + dx) * 4) as usize;

            let sa = src.data[sidx + 3] as f32 / 255.0;
            if sa <= 0.0 {
                continue;
            }

            let da = dst.data[didx + 3] as f32 / 255.0;

            let out_a = sa + da * (1.0 - sa);
            if out_a <= 0.0 {
                dst.data[didx + 3] = 0;
                continue;
            }

            for c in 0..3 {
                let sc = src.data[sidx + c] as f32 / 255.0;
                let dc = dst.data[didx + c] as f32 / 255.0;
                let out_c = (sc * sa + dc * da * (1.0 - sa)) / out_a;
                dst.data[didx + c] = (out_c * 255.0).round().clamp(0.0, 255.0) as u8;
            }
            dst.data[didx + 3] = (out_a * 255.0).round().clamp(0.0, 255.0) as u8;
        }
    }
}

fn rotate_rgba_bilinear(src: &ImageAsset, degrees: f32) -> ImageAsset {
    let w = src.width as f32;
    let h = src.height as f32;

    let theta = degrees.to_radians();
    let cos_t = theta.cos();
    let sin_t = theta.sin();

    let new_w = (w.abs() * cos_t.abs() + h.abs() * sin_t.abs())
        .ceil()
        .max(1.0) as u32;
    let new_h = (w.abs() * sin_t.abs() + h.abs() * cos_t.abs())
        .ceil()
        .max(1.0) as u32;

    let mut out = ImageAsset {
        width: new_w,
        height: new_h,
        data: vec![0u8; (new_w * new_h * 4) as usize],
    };

    let src_cx = (w - 1.0) * 0.5;
    let src_cy = (h - 1.0) * 0.5;
    let dst_cx = (new_w as f32 - 1.0) * 0.5;
    let dst_cy = (new_h as f32 - 1.0) * 0.5;

    for y in 0..new_h {
        for x in 0..new_w {
            let dx = x as f32 - dst_cx;
            let dy = y as f32 - dst_cy;

            // Inverse rotate
            let sx = dx * cos_t + dy * sin_t + src_cx;
            let sy = -dx * sin_t + dy * cos_t + src_cy;

            let sample = bilinear_sample(src, sx, sy);
            let idx = ((y * new_w + x) * 4) as usize;
            out.data[idx..idx + 4].copy_from_slice(&sample);
        }
    }

    out
}

fn bilinear_sample(img: &ImageAsset, x: f32, y: f32) -> [u8; 4] {
    let w = img.width as i32;
    let h = img.height as i32;

    let x0 = x.floor() as i32;
    let y0 = y.floor() as i32;
    let x1 = x0 + 1;
    let y1 = y0 + 1;

    if x0 < 0 || y0 < 0 || x0 >= w || y0 >= h {
        return [0, 0, 0, 0];
    }

    let fx = x - x0 as f32;
    let fy = y - y0 as f32;

    let p00 = get_px(img, x0, y0);
    let p10 = if x1 < w { get_px(img, x1, y0) } else { p00 };
    let p01 = if y1 < h { get_px(img, x0, y1) } else { p00 };
    let p11 = if x1 < w && y1 < h {
        get_px(img, x1, y1)
    } else {
        p00
    };

    let mut out = [0u8; 4];
    for c in 0..4 {
        let v00 = p00[c] as f32;
        let v10 = p10[c] as f32;
        let v01 = p01[c] as f32;
        let v11 = p11[c] as f32;

        let v0 = v00 + (v10 - v00) * fx;
        let v1 = v01 + (v11 - v01) * fx;
        let v = v0 + (v1 - v0) * fy;
        out[c] = v.round().clamp(0.0, 255.0) as u8;
    }
    out
}

fn get_px(img: &ImageAsset, x: i32, y: i32) -> [u8; 4] {
    let idx = ((y as u32 * img.width + x as u32) * 4) as usize;
    [
        img.data[idx],
        img.data[idx + 1],
        img.data[idx + 2],
        img.data[idx + 3],
    ]
}

fn draw_filled_circle(img: &mut ImageAsset, cx: i32, cy: i32, radius: i32, rgba: (u8, u8, u8, u8)) {
    if radius <= 0 {
        return;
    }

    let (r, g, b, a) = rgba;
    let w = img.width as i32;
    let h = img.height as i32;
    let r2 = radius * radius;

    for y in (cy - radius)..=(cy + radius) {
        if y < 0 || y >= h {
            continue;
        }
        for x in (cx - radius)..=(cx + radius) {
            if x < 0 || x >= w {
                continue;
            }
            let dx = x - cx;
            let dy = y - cy;
            if dx * dx + dy * dy <= r2 {
                let idx = ((y as u32 * img.width + x as u32) * 4) as usize;
                // Overwrite since shadow surface is otherwise transparent.
                img.data[idx] = r;
                img.data[idx + 1] = g;
                img.data[idx + 2] = b;
                img.data[idx + 3] = a;
            }
        }
    }
}

/// Small deterministic RNG to avoid adding dependencies.
#[derive(Clone, Debug)]
pub struct SimpleRng(u64);

impl SimpleRng {
    pub fn new(seed: u64) -> Self {
        Self(seed.max(1))
    }

    fn next_u64(&mut self) -> u64 {
        // xorshift64*
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(2685821657736338717)
    }

    pub fn gen_f32(&mut self) -> f32 {
        let v = (self.next_u64() >> 40) as u32;
        (v as f32) / ((1u32 << 24) as f32)
    }

    pub fn gen_usize(&mut self, upper: usize) -> usize {
        if upper <= 1 {
            return 0;
        }
        (self.next_u64() as usize) % upper
    }
}
