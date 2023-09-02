use crate::ClientState;
use bevy::prelude::*;
use bevy::render::render_resource::{AddressMode, FilterMode, SamplerDescriptor};
use bevy::render::texture::ImageSampler;

pub const BLOCK_TEXTURE_ROWS: u8 = 8;
pub const BLOCK_TEXTURE_COLUMNS: u8 = 16;

pub const UV_WIDTH: f32 = 1.0 / BLOCK_TEXTURE_COLUMNS as f32;
pub const UV_HEIGHT: f32 = 1.0 / BLOCK_TEXTURE_ROWS as f32;
pub type UvCoordinate = [Vec2; 4];

#[derive(Resource)]
pub struct ResourcePack {
    pub handle: Handle<StandardMaterial>,
}

#[derive(Resource, Default)]
struct TexturePackLoading(Handle<Image>);

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

fn setup_texture(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut loading: ResMut<TexturePackLoading>,
) {
    let custom_texture_handle: Handle<Image> = asset_server.load("textures/spritesheet_blocks.png");

    *loading = TexturePackLoading(custom_texture_handle.clone());

    let resource_pack = materials.add(StandardMaterial {
        base_color_texture: Some(custom_texture_handle),
        unlit: true,
        ..Default::default()
    });

    commands.insert_resource(ResourcePack {
        handle: resource_pack,
    });
}

fn check_assets_ready(
    mut commands: Commands,
    server: Res<AssetServer>,
    loading: Res<TexturePackLoading>,
    mut next_state: ResMut<NextState<ClientState>>,
    mut images: ResMut<Assets<Image>>,
) {
    use bevy::asset::LoadState;

    match server.get_load_state(loading.0.clone()) {
        LoadState::Loaded => {
            let image = images.get_mut(&loading.0).unwrap();
            image.sampler_descriptor = ImageSampler::Descriptor(SamplerDescriptor {
                mag_filter: FilterMode::Nearest,
                min_filter: FilterMode::Nearest,
                mipmap_filter: FilterMode::Nearest,
                address_mode_u: AddressMode::ClampToBorder,
                address_mode_v: AddressMode::ClampToBorder,
                address_mode_w: AddressMode::ClampToBorder,
                ..default()
            });

            commands.remove_resource::<TexturePackLoading>();
            next_state.set(ClientState::MainMenu);
        }
        _ => {
            // NotLoaded/Loading: not fully ready yet
        }
    }
}

pub struct TexturePlugin;
impl Plugin for TexturePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TexturePackLoading>()
            .add_systems(
                Update,
                check_assets_ready.run_if(in_state(ClientState::LoadingTexture)),
            )
            .add_systems(Startup, setup_texture);
    }
}
