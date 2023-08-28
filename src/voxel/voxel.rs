#[derive(Copy, Clone, Debug, PartialEq)]
pub enum VoxelType {
    Void,
    Grass,
    Dirt,
    Water,
    Stone,
    Wood,
    Sand
}

#[derive(Copy, Clone, Debug)]
pub struct Voxel {
    voxel_type: VoxelType
}

impl Default for Voxel {
    fn default() -> Self {
        Self {voxel_type: VoxelType::Void}
    }
}

impl Voxel {
    pub fn new(voxel_type: VoxelType) -> Self {
        Self { voxel_type }
    }

    pub fn new_empty() -> Self {
        Self { voxel_type: VoxelType::Void }
    }

    pub fn is_solid(&self) -> bool {
        self.voxel_type != VoxelType::Void
    }

    pub fn set_type(&mut self, voxel_type: VoxelType) {
        self.voxel_type = voxel_type;
    }
}