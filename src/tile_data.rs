use crate::{Direction, Triangle};

#[derive(Debug)]
pub struct TileData {
    pub model: Vec<Triangle>,
    pub weight: u32,
    pub up_edge: String,
    pub right_edge: String,
    pub down_edge: String,
    pub left_edge: String,
    pub up_edge_suffix: Option<String>,
    pub right_edge_suffix: Option<String>,
    pub down_edge_suffix: Option<String>,
    pub left_edge_suffix: Option<String>,
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

    pub fn get_suffix(&self, direction: Direction) -> Option<&str> {
        match direction {
            Direction::Up => self.up_edge_suffix.as_deref(),
            Direction::Right => self.right_edge_suffix.as_deref(),
            Direction::Down => self.down_edge_suffix.as_deref(),
            Direction::Left => self.left_edge_suffix.as_deref(),
        }
    }

    pub fn check_edge(&self, direction: Direction, edge: &str, suffix: Option<&str>) -> bool {
        let own_suffix = self.get_suffix(direction);
        if let (Some(suffix), Some(own_suffix)) = (suffix, own_suffix) {
            self.get_edge(direction) == edge && suffix != own_suffix // Prevent matching tile with itself rotated
        } else {
            self.get_edge(direction) == edge
        }
    }
}
