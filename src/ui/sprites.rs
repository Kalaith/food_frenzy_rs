//! Interim art atlas over `assets/images/interior_sheet.png`.
//!
//! The sheet is an irregular pixel-art set (floors, appliances, furniture,
//! decor, a chef). Each [`Region`] names a source rect in the sheet; helpers
//! blit a region into a destination rect, contain-fit it to a height, or tile
//! it across an area. Rendering is view-only — no gameplay state is touched.

use macroquad::prelude::*;

/// A named source sprite in `interior_sheet.png`, as `[x, y, w, h]` pixels.
#[derive(Clone, Copy)]
pub enum Region {
    FloorWoodDark,
    RoundTable,
    ChairWood,
    Stove,
    PotStove,
    PrepVeg,
    Mixer,
    Sink,
    SignFeast,
    Plant,
    MenuBoard,
    StringLights,
    FramedPic,
    Candles,
    Chef,
}

impl Region {
    /// Source rect `[x, y, w, h]` in sheet pixels.
    const fn px(self) -> [f32; 4] {
        match self {
            Region::FloorWoodDark => [112.0, 0.0, 112.0, 110.0],
            Region::RoundTable => [7.0, 448.0, 112.0, 119.0],
            Region::ChairWood => [922.0, 432.0, 71.0, 135.0],
            Region::Stove => [5.0, 150.0, 192.0, 161.0],
            Region::PotStove => [795.0, 137.0, 189.0, 174.0],
            Region::PrepVeg => [381.0, 163.0, 196.0, 148.0],
            Region::Mixer => [590.0, 146.0, 189.0, 165.0],
            Region::Sink => [999.0, 142.0, 203.0, 169.0],
            Region::SignFeast => [1230.0, 250.0, 80.0, 54.0],
            Region::Plant => [340.0, 573.0, 97.0, 114.0],
            Region::MenuBoard => [990.0, 569.0, 86.0, 115.0],
            Region::StringLights => [1170.0, 574.0, 168.0, 53.0],
            Region::FramedPic => [667.0, 569.0, 92.0, 82.0],
            Region::Candles => [1087.0, 611.0, 79.0, 76.0],
            Region::Chef => [1258.0, 664.0, 71.0, 120.0],
        }
    }

    fn source(self) -> Rect {
        let [x, y, w, h] = self.px();
        Rect::new(x, y, w, h)
    }

    /// Native pixel size of the sprite.
    pub fn size(self) -> Vec2 {
        let [_, _, w, h] = self.px();
        vec2(w, h)
    }
}

/// Draw a region stretched to fill `dest`.
pub fn blit(sheet: &Texture2D, region: Region, dest: Rect) {
    draw_texture_ex(
        sheet,
        dest.x,
        dest.y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(dest.w, dest.h)),
            source: Some(region.source()),
            ..Default::default()
        },
    );
}

/// Draw a region contain-fit to `target_h`, its bottom-centre anchored at
/// `(cx, baseline_y)` — the natural placement for furniture and characters
/// standing on the floor.
pub fn blit_grounded(sheet: &Texture2D, region: Region, cx: f32, baseline_y: f32, target_h: f32) {
    let size = region.size();
    let scale = target_h / size.y;
    let w = size.x * scale;
    blit(
        sheet,
        region,
        Rect::new(cx - w * 0.5, baseline_y - target_h, w, target_h),
    );
}

/// Tile a region across `area` at `cell` pixels, clipping the final row/column
/// to the area bounds. Used to lay the dining-room floor.
pub fn tile(sheet: &Texture2D, region: Region, area: Rect, cell: f32) {
    let src = region.source();
    let mut y = area.y;
    while y < area.y + area.h {
        let ch = cell.min(area.y + area.h - y);
        let mut x = area.x;
        while x < area.x + area.w {
            let cw = cell.min(area.x + area.w - x);
            let sx = src.w * (cw / cell);
            let sy = src.h * (ch / cell);
            draw_texture_ex(
                sheet,
                x,
                y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(cw, ch)),
                    source: Some(Rect::new(src.x, src.y, sx, sy)),
                    ..Default::default()
                },
            );
            x += cell;
        }
        y += cell;
    }
}
