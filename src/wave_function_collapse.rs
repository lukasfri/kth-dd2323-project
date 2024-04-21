use std::{
    collections::{HashSet, VecDeque},
    f32::consts::PI,
    fs::File,
    io::{BufRead, BufReader},
};

use anyhow::{bail, ensure};
use nalgebra::{Rotation3, Vector2};

use crate::{model_loader::ModelLoader, scene::Scene, tile::Tile, tile_data::TileData, Direction};

pub enum PlacementStrategy {
    Random,
    Growing,
    Ordered,
    LeastEntropy,
}

pub struct WFC<'a> {
    map_size: usize,
    scene: &'a mut Scene,
    max_placement_iterations: u32,
}

impl<'a> WFC<'a> {
    pub fn new(scene: &'a mut Scene, map_size: usize) -> Self {
        WFC {
            map_size,
            scene,
            max_placement_iterations: 500,
        }
    }

    // Where the actual Wave Function Collapse logic happens
    pub fn place_tiles(&mut self, placement_strategy: &PlacementStrategy) -> anyhow::Result<()> {
        // TODO: continue until filled map
        match self.load_tiles() {
            Ok(tile_datas) => {
                let possible_tiles: Vec<&TileData> = tile_datas.iter().collect();

                // Fill tiles list with all possibilities
                let mut tiles: Vec<Tile> = vec![];
                for i in 0..(self.map_size * self.map_size) {
                    tiles.push(Tile::new(possible_tiles.clone(), self.index1dto2d(i)));
                }
                // Set of indexes of tiles that haven't been collapsed
                let mut uncollapsed_tiles: HashSet<usize> =
                    (0..(self.map_size * self.map_size)).collect::<HashSet<usize>>();
                let mut iterations = 0;

                match placement_strategy {
                    PlacementStrategy::Random => self.random_placement_strategy(
                        &mut tiles,
                        &mut uncollapsed_tiles,
                        &mut iterations,
                    ),
                    PlacementStrategy::Growing => self.growing_placement_strategy(
                        &mut tiles,
                        &mut uncollapsed_tiles,
                        &mut iterations,
                    ),
                    PlacementStrategy::Ordered => self.ordered_placement_strategy(
                        &mut tiles,
                        &mut uncollapsed_tiles,
                        &mut iterations,
                    ),
                    PlacementStrategy::LeastEntropy => self.least_entropy_placement_strategy(
                        &mut tiles,
                        &mut uncollapsed_tiles,
                        &mut iterations,
                    ),
                }

                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    fn collapse_tile(
        &mut self,
        tiles: &mut [Tile],
        uncollapsed_tiles: &mut HashSet<usize>,
        tile_index: usize,
    ) {
        if tiles[tile_index].collapse(self.scene) {
            uncollapsed_tiles.remove(&tile_index);

            // Update neighbours
            for direction in Direction::iterator() {
                let neighbour_position = Vector2::<i32>::new(
                    tiles[tile_index].tile_position.x as i32,
                    tiles[tile_index].tile_position.y as i32,
                ) + direction.get_vector();
                if self.within_grid(neighbour_position) {
                    let neighbour_index = self.index2dto1d(Vector2::<usize>::new(
                        neighbour_position.x as usize,
                        neighbour_position.y as usize,
                    ));
                    if let Some(tile_data) = tiles[tile_index].data {
                        if tiles[neighbour_index]
                            .remove_options(direction.get_opposite(), tile_data.get_edge(direction))
                        {
                            // TODO: backtrack?
                            uncollapsed_tiles.remove(&neighbour_index);
                        }
                    }
                }
            }
        }
    }

    fn collapse_center_tile(&mut self, tiles: &mut [Tile], uncollapsed_tiles: &mut HashSet<usize>) {
        let center_tile_index =
            self.index2dto1d(Vector2::<usize>::new(self.map_size / 2, self.map_size / 2));
        self.collapse_tile(tiles, uncollapsed_tiles, center_tile_index);
        uncollapsed_tiles.remove(&center_tile_index);
    }

    fn load_tiles(&self) -> anyhow::Result<Vec<TileData>> {
        let mut tiles: Vec<TileData> = vec![];
        let model_loader = ModelLoader::new();

        match File::open("./config.txt") {
            Ok(file) => {
                let reader = BufReader::new(file);
                for (index, line) in reader.lines().enumerate() {
                    match line {
                        Ok(line) => {
                            // Ignore comments
                            if line.starts_with('#') {
                                continue;
                            }

                            // Validate inputs
                            let values = line
                                .replace(' ', "")
                                .split(',')
                                .map(|s| s.to_string())
                                .collect::<Vec<String>>();
                            ensure!(
                                values.len() == 7,
                                format!(
                                    "Line {} \"{}\" does not contain all values required",
                                    index + 1,
                                    line
                                )
                            );
                            let weight = values[1].parse::<u32>();
                            ensure!(
                                weight.is_ok() && weight.clone().unwrap() > 0,
                                format!(
                                    "On line {} the weight value can only be a non-negative integer",
                                    index + 1
                                )
                            );
                            let weight = weight.unwrap();
                            ensure!(
                                values[6] == "1" || values[6] == "2" || values[6] == "4",
                                format!(
                                    "On line {} the rotatable value can only be 1, 2 or 4",
                                    index + 1
                                )
                            );

                            // Load models
                            let rotation_angles: Vec<Rotation3<f32>> = match values[6].as_str() {
                                "4" => vec![
                                    Rotation3::from_euler_angles(0.0, 0.0, 0.0),
                                    Rotation3::from_euler_angles(0.0, 0.0, PI / 2.0),
                                    Rotation3::from_euler_angles(0.0, 0.0, PI),
                                    Rotation3::from_euler_angles(0.0, 0.0, 3.0 / 2.0 * PI),
                                ],
                                "2" => vec![
                                    Rotation3::from_euler_angles(0.0, 0.0, 0.0),
                                    Rotation3::from_euler_angles(0.0, 0.0, PI / 2.0),
                                ],
                                _ => vec![Rotation3::from_euler_angles(0.0, 0.0, 0.0)],
                            };

                            // Store each rotation as seperate model
                            for (index, rotation) in rotation_angles.into_iter().enumerate() {
                                match model_loader
                                    .load_gltf_model(format!("assets/{}", &values[0]), rotation)
                                {
                                    Ok(model) => {
                                        let tile = TileData {
                                            model,
                                            weight,
                                            up_edge: values[2 + (index % 4)].clone(),
                                            right_edge: values[2 + ((index + 1) % 4)].clone(),
                                            down_edge: values[2 + ((index + 2) % 4)].clone(),
                                            left_edge: values[2 + ((index + 3) % 4)].clone(),
                                        };
                                        tiles.push(tile);
                                    }
                                    Err(err) => return Err(err),
                                }
                            }
                        }
                        Err(err) => return Err(anyhow::Error::from(err)),
                    }
                }
            }
            Err(_) => {
                bail!("Could not find config file config.txt")
            }
        }
        Ok(tiles)
    }

    fn index2dto1d(&self, index: Vector2<usize>) -> usize {
        index.y * self.map_size + index.x
    }

    fn index1dto2d(&self, index: usize) -> Vector2<usize> {
        Vector2::<usize>::new(index % self.map_size, index / self.map_size)
    }

    fn within_grid(&self, index: Vector2<i32>) -> bool {
        index.x >= 0
            && index.y >= 0
            && index.x < self.map_size as i32
            && index.y < self.map_size as i32
    }

    /** PLACEMENT STRATEGIES **/
    // Chooses random tile to collapse
    fn random_placement_strategy(
        &mut self,
        tiles: &mut [Tile],
        uncollapsed_tiles: &mut HashSet<usize>,
        iterations: &mut u32,
    ) {
        self.collapse_center_tile(tiles, uncollapsed_tiles);

        while *iterations < self.max_placement_iterations && !uncollapsed_tiles.is_empty() {
            let choosen_tile = *uncollapsed_tiles.iter().next().expect("Set is not empty");
            uncollapsed_tiles.remove(&choosen_tile);
            self.collapse_tile(tiles, uncollapsed_tiles, choosen_tile);
            *iterations += 1;
        }
    }

    // Collapses tiles in the order of a BFS from the starting tile
    fn growing_placement_strategy(
        &mut self,
        tiles: &mut [Tile],
        uncollapsed_tiles: &mut HashSet<usize>,
        iterations: &mut u32,
    ) {
        let mut tiles_queue = VecDeque::<usize>::new();
        tiles_queue.push_back(
            self.index2dto1d(Vector2::<usize>::new(self.map_size / 2, self.map_size / 2)),
        );

        while !tiles_queue.is_empty()
            && *iterations < self.max_placement_iterations
            && !uncollapsed_tiles.is_empty()
        {
            let choosen_tile = tiles_queue.pop_front().expect("Queue is not empty");
            uncollapsed_tiles.remove(&choosen_tile);
            self.collapse_tile(tiles, uncollapsed_tiles, choosen_tile);

            // Add neighbours to queue
            for direction in Direction::iterator() {
                let neighbour_position = Vector2::<i32>::new(
                    tiles[choosen_tile].tile_position.x as i32,
                    tiles[choosen_tile].tile_position.y as i32,
                ) + direction.get_vector();
                if self.within_grid(neighbour_position) {
                    let neighbour_index = self.index2dto1d(Vector2::<usize>::new(
                        neighbour_position.x as usize,
                        neighbour_position.y as usize,
                    ));
                    let neighbour_tile = &tiles[neighbour_index];

                    // Check that tile hasn't collapsed
                    if neighbour_tile.data.is_none() {
                        tiles_queue.push_back(neighbour_index);
                    }
                }
            }

            *iterations += 1;
        }
    }

    // Collapese tiles in an order of left to right, down to up
    fn ordered_placement_strategy(
        &mut self,
        tiles: &mut [Tile],
        uncollapsed_tiles: &mut HashSet<usize>,
        iterations: &mut u32,
    ) {
        for tile_index in 0..(self.map_size * self.map_size) {
            if *iterations > self.max_placement_iterations {
                return;
            }

            uncollapsed_tiles.remove(&tile_index);
            self.collapse_tile(tiles, uncollapsed_tiles, tile_index);
            *iterations += 1;
        }
    }

    // Collapses the tiles in the order of least entropy first
    fn least_entropy_placement_strategy(
        &mut self,
        tiles: &mut [Tile],
        uncollapsed_tiles: &mut HashSet<usize>,
        iterations: &mut u32,
    ) {
        self.collapse_center_tile(tiles, uncollapsed_tiles);

        while *iterations < self.max_placement_iterations && !uncollapsed_tiles.is_empty() {
            // Find element with least entropy
            let mut least_entropy = usize::MAX;
            let mut least_entropy_index = 0;

            for tile_index in uncollapsed_tiles.iter() {
                let tile_entropy = tiles[*tile_index].possible_tiles.len();
                if tile_entropy < least_entropy {
                    least_entropy = tile_entropy;
                    least_entropy_index = *tile_index;
                }
            }

            uncollapsed_tiles.remove(&least_entropy_index);
            self.collapse_tile(tiles, uncollapsed_tiles, least_entropy_index);
            *iterations += 1;
        }
    }
}
