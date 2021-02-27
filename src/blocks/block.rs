use crate::blocks::block_type::BlockType;
use crate::blocks::blockside::BlockSides;
use crate::constants::COLORS;
use crate::positions::{GlobalBlockPos, ObjectPos};
use crate::renderer::vertex::{vertex, vertex_typed, Vertex};
use rand::distributions::{Distribution, Standard};
use rand::Rng;
use serde::{Deserialize, Serialize};

pub type BlockId = u8;

#[inline]
pub fn get_blocktype(block_id: BlockId) -> BlockType {
    match block_id {
        0 => BlockType::Grass,
        1 => BlockType::Water,
        2 => BlockType::Dirt,
        3 => BlockType::Stone,
        4 => BlockType::Sand,
        5 => BlockType::Air,
        6 => BlockType::Leaf,
        _ => BlockType::Unknown,
    }
}
#[inline]
pub fn get_blockid(block: BlockType) -> BlockId {
    match block {
        BlockType::Grass => 0,
        BlockType::Water => 1,
        BlockType::Dirt => 2,
        BlockType::Stone => 3,
        BlockType::Sand => 4,
        BlockType::Air => 5,
        BlockType::Leaf => 6,
        BlockType::Unknown => 255,
    }
}
#[inline]
pub fn should_render_against(source_block_id: BlockId, neighbor_block_id: BlockId) -> bool {
    if source_block_id == neighbor_block_id {
        return false;
    }
    if COLORS[neighbor_block_id as usize][3] == 255.0 {
        return false;
    }
    return true;
}
pub fn get_mesh(
    block_id: BlockId,
    pos: &GlobalBlockPos,
    sides: &BlockSides,
) -> (Vec<Vertex>, Vec<u32>) {
    crate::blocks::block_mesh::get_mesh(block_id, pos, sides)
}
