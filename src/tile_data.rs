use crate::{Direction, Triangle};

#[derive(Debug)]
pub struct TileData {
    pub model: Vec<Triangle>,
    pub weight: u32,
    pub up_edge: String,
    pub right_edge: String,
    pub down_edge: String,
    pub left_edge: String,
}

impl TileData {
    pub fn get_edge(&self, direction: Direction) -> &str {
        match direction {
            Direction::Up => self.up_edge.as_str(),
            Direction::Right => self.right_edge.as_str(),
            Direction::Down => self.down_edge.as_str(),
            Direction::Left => self.left_edge.as_str(),
        }
    }

    pub fn check_edge(&self, direction: Direction, edge: &str) -> bool {
        self.get_edge(direction) == edge
    }
}
