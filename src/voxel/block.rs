use crate::voxel::direction::Direction;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BlockType {
    Void,
    Grass,
    Dirt,
    Stone,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Block {
    pub(crate) voxel_type: BlockType,
}

impl Default for Block {
    fn default() -> Self {
        Self {
            voxel_type: BlockType::Void,
        }
    }
}

impl Block {
    pub fn new_empty() -> Self {
        Self {
            voxel_type: BlockType::Void,
        }
    }

    pub fn is_solid(&self) -> bool {
        self.voxel_type != BlockType::Void
    }

    pub fn set_type(&mut self, voxel_type: BlockType) {
        self.voxel_type = voxel_type;
    }

    pub fn get_face(voxel_type: &BlockType, direction: &Direction) -> u16 {
        match voxel_type {
            BlockType::Void => 0,
            BlockType::Grass => match direction {
                Direction::Up => 23,
                Direction::Down => 9,
                _ => 10,
            },
            BlockType::Dirt => 9,
            BlockType::Stone => 50,
        }
    }
}
