use super::*;

#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
#[packet(id = packet_id::DIFFICULTY_S2C)]
pub struct DifficultyS2c {
    pub difficulty: Difficulty,
    pub locked: bool,
}
