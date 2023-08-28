use crate::voxel::direction::Direction;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum VoxelType {
    Void,
    Grass,
    Dirt,
    Stone,
}

#[derive(Copy, Clone, Debug)]
pub struct Voxel {
    pub(crate) voxel_type: VoxelType,
}

impl Default for Voxel {
    fn default() -> Self {
        Self {
            voxel_type: VoxelType::Void,
        }
    }
}

impl Voxel {
    pub fn new(voxel_type: VoxelType) -> Self {
        Self { voxel_type }
    }

    pub fn new_empty() -> Self {
        Self {
            voxel_type: VoxelType::Void,
        }
    }

    pub fn is_solid(&self) -> bool {
        self.voxel_type != VoxelType::Void
    }

    pub fn set_type(&mut self, voxel_type: VoxelType) {
        self.voxel_type = voxel_type;
    }

    pub fn get_face(voxel_type: &VoxelType, direction: &Direction) -> u16 {
        match voxel_type {
            VoxelType::Void => 0,
            VoxelType::Grass => match direction {
                Direction::Up => 23,
                Direction::Down => 9,
                _ => 10,
            },
            VoxelType::Dirt => 9,
            VoxelType::Stone => 50,
        }
    }
}
