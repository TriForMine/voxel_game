use bevy::prelude::Vec2;

pub const BLOCK_TEXTURE_ROWS: u8 = 8;
pub const BLOCK_TEXTURE_COLUMNS: u8 = 16;

pub const UV_WIDTH: f32 = 1.0 / BLOCK_TEXTURE_COLUMNS as f32;
pub const UV_HEIGHT: f32 = 1.0 / BLOCK_TEXTURE_ROWS as f32;
pub type UvCoordinate = [Vec2; 4];

pub fn convert_face_id_to_uv(face_id: u16) -> UvCoordinate {
    let row = (face_id as u8) / BLOCK_TEXTURE_COLUMNS;
    let col = (face_id as u8) % BLOCK_TEXTURE_COLUMNS;

    let min_u = col as f32 * UV_WIDTH;
    let max_u = min_u + UV_WIDTH;
    let min_v = row as f32 * UV_HEIGHT;
    let max_v = min_v + UV_HEIGHT;

    [
        Vec2::new(min_u, min_v),
        Vec2::new(max_u, min_v),
        Vec2::new(max_u, max_v),
        Vec2::new(min_u, max_v),
    ]
}
