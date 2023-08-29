use crate::voxel::chunk::{Chunk, ChunkData, SIZE};
use crate::voxel::voxel::VoxelType;
use bevy::math::IVec3;
use once_cell::sync::Lazy;
use std::sync::RwLock;

pub static TERRAIN_GENERATOR: Lazy<RwLock<TerrainGenerator>> = Lazy::new(Default::default);

#[derive(Default)]
pub struct TerrainGenerator {
    seed: i32,
}

impl TerrainGenerator {
    pub fn generate(&self, chunk_pos: IVec3, buffer: &mut ChunkData) {
        if buffer.is_empty() {
            return;
        }

        let chunk_world_pos = chunk_pos * IVec3::new(SIZE, 0, SIZE);

        use simdnoise::NoiseBuilder;
        let (noise, _min, _max) = NoiseBuilder::gradient_2d_offset(
            chunk_world_pos.x as f32,
            SIZE.try_into().unwrap(),
            chunk_world_pos.z as f32,
            SIZE.try_into().unwrap(),
        )
        .with_freq(0.008)
        .with_seed(self.seed)
        .generate();

        for x in 0..(SIZE) {
            for z in 0..(SIZE) {
                let height: i32 =
                    (42.0 + noise[(z * (SIZE) + x) as usize] * 64.0 * 8.0).round() as i32;

                for y in 0..height {
                    let voxel = buffer
                        .get_mut(Chunk::get_index(&IVec3::new(x, y, z)))
                        .unwrap();
                    if y == (height - 1) {
                        voxel.set_type(VoxelType::Grass);
                    } else if y < height && y > height - 3 {
                        voxel.set_type(VoxelType::Dirt);
                    } else if y < height {
                        voxel.set_type(VoxelType::Stone);
                    }
                }
            }
        }
    }
}
