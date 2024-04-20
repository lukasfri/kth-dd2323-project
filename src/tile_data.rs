use crate::Triangle;

pub struct TileData {
    pub model: Vec<Triangle>,
    pub up_edge: String,
    pub right_edge: String,
    pub down_edge: String,
    pub left_edge: String,
}
