use std::{
    collections::{BTreeSet, VecDeque},
    f32::consts::PI,
    fs::File,
    io::{BufRead, BufReader},
};

use anyhow::{bail, ensure};
use nalgebra::{Rotation3, Vector2};
use rand::{rngs::StdRng, seq::IteratorRandom, SeedableRng};

use crate::{model_loader::ModelLoader, scene::Scene, tile::Tile, tile_data::TileData, Direction};

pub enum PlacementStrategy {
    Random,
    Growing,
    Ordered,
    LeastEntropy,
}

pub struct WFC<'a> {
    scene: &'a mut Scene,
}

impl<'a> WFC<'a> {
    pub fn new(scene: &'a mut Scene) -> Self {
        WFC { scene }
    }

    // Where the actual Wave Function Collapse logic happens
    pub fn place_tiles(&mut self) -> anyhow::Result<()> {
        let (placement_strategy, map_size, max_iterations, mut random, tile_datas) =
            self.load_tiles()?;

        let possible_tiles: Vec<&TileData> = tile_datas.iter().collect();

        // Fill tiles list with all possibilities
        let mut tiles: Vec<Tile> = vec![];
        for i in 0..(map_size * map_size) {
            tiles.push(Tile::new(
                possible_tiles.clone(),
                self.index1dto2d(i, map_size),
            ));
        }
        // Set of indexes of tiles that haven't been collapsed
        let mut uncollapsed_tiles: BTreeSet<usize> =
            (0..(map_size * map_size)).collect::<BTreeSet<usize>>();
        let mut iterations = 0;

        match placement_strategy {
            PlacementStrategy::Random => self.random_placement_strategy(
                &mut tiles,
                &mut uncollapsed_tiles,
                &mut iterations,
                map_size,
                max_iterations,
                &mut random,
            ),
            PlacementStrategy::Growing => self.growing_placement_strategy(
                &mut tiles,
                &mut uncollapsed_tiles,
                &mut iterations,
                map_size,
                max_iterations,
                &mut random,
            ),
            PlacementStrategy::Ordered => self.ordered_placement_strategy(
                &mut tiles,
                &mut uncollapsed_tiles,
                &mut iterations,
                map_size,
                max_iterations,
                &mut random,
            ),
            PlacementStrategy::LeastEntropy => self.least_entropy_placement_strategy(
                &mut tiles,
                &mut uncollapsed_tiles,
                &mut iterations,
                map_size,
                max_iterations,
                &mut random,
            ),
        }

        Ok(())
    }

    fn collapse_tile(
        &mut self,
        tiles: &mut [Tile],
        uncollapsed_tiles: &mut BTreeSet<usize>,
        tile_index: usize,
        random: &mut StdRng,
        map_size: usize,
    ) {
        if !tiles[tile_index].collapse(self.scene, random) {
            return;
        }

        uncollapsed_tiles.remove(&tile_index);

        // Update neighbours
        for direction in Direction::iterator() {
            let neighbour_position = Vector2::<i32>::new(
                tiles[tile_index].tile_position.x as i32,
                tiles[tile_index].tile_position.y as i32,
            ) + direction.get_vector();
            if !self.within_grid(neighbour_position, map_size) {
                continue;
            }

            let neighbour_index = self.index2dto1d(
                Vector2::<usize>::new(neighbour_position.x as usize, neighbour_position.y as usize),
                map_size,
            );
            if let Some(tile_data) = tiles[tile_index].data {
                if tiles[neighbour_index]
                    .remove_options(direction.get_opposite(), tile_data.get_edge(direction))
                {
                    uncollapsed_tiles.remove(&neighbour_index);
                }
            }
        }
    }

    fn collapse_center_tile(
        &mut self,
        tiles: &mut [Tile],
        uncollapsed_tiles: &mut BTreeSet<usize>,
        random: &mut StdRng,
        map_size: usize,
    ) {
        let center_tile_index =
            self.index2dto1d(Vector2::<usize>::new(map_size / 2, map_size / 2), map_size);
        self.collapse_tile(
            tiles,
            uncollapsed_tiles,
            center_tile_index,
            random,
            map_size,
        );
        uncollapsed_tiles.remove(&center_tile_index);
    }

    fn load_tiles(
        &self,
    ) -> anyhow::Result<(&PlacementStrategy, usize, u32, StdRng, Vec<TileData>)> {
        let mut placement_strategy: &PlacementStrategy = &PlacementStrategy::LeastEntropy;
        let mut map_size: usize = 10;
        let mut max_iterations: u32 = 1000;
        let mut tileset_path: String = "".to_owned();
        let mut seed: u64 = 0;

        self.read_config_file(
            &mut placement_strategy,
            &mut map_size,
            &mut max_iterations,
            &mut tileset_path,
            &mut seed,
        )?;

        let tiles = self.read_tileset_config_file(&mut tileset_path)?;

        Ok((
            placement_strategy,
            map_size,
            max_iterations,
            if seed == 0 {
                StdRng::from_entropy()
            } else {
                StdRng::seed_from_u64(seed)
            },
            tiles,
        ))
    }

    fn read_tileset_config_file(&self, tileset_path: &mut String) -> anyhow::Result<Vec<TileData>> {
        let mut tiles: Vec<TileData> = vec![];
        let model_loader = ModelLoader::new();

        let file = File::open(format!("{}/tiles_config.txt", tileset_path)).map_err(|_| {
            anyhow::format_err!(
                "Could not find config file {}/tiles_config.txt",
                tileset_path
            )
        })?;

        let reader = BufReader::new(file);
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
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
                    "On line {} the weight value can only be a positive integer",
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
                let model = model_loader
                    .load_gltf_model(format!("{}/{}", tileset_path, &values[0]), rotation)?;
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
        }

        Ok(tiles)
    }

    fn read_config_file(
        &self,
        placement_strategy: &mut &PlacementStrategy,
        map_size: &mut usize,
        max_iterations: &mut u32,
        tileset_path: &mut String,
        seed: &mut u64,
    ) -> anyhow::Result<()> {
        const CONFIG_FILE_PATH: &str = "./config.txt";

        let file = File::open(CONFIG_FILE_PATH)
            .map_err(|_| anyhow::format_err!("Could not find config file {}", CONFIG_FILE_PATH))?;

        // Read values
        let reader = BufReader::new(file);
        for (index, line) in reader.lines().enumerate() {
            let line = line?;

            // Ignore comments
            if line.starts_with('#') {
                continue;
            }
            let parts = line
                .replace(' ', "")
                .split('=')
                .map(|s| s.to_string())
                .collect::<Vec<String>>();

            ensure!(parts.len() == 2 && parts[0].as_str() != "" && parts[1].as_str() != "", format!("Error in {} on line {}. The config file accepts lines in the format of KEY=VALUE", CONFIG_FILE_PATH, index + 1));

            // Read and validate options
            match parts[0].as_str() {
                "placement_strategy" => {
                    match parts[1].as_str() {
                        "least_entropy" => *placement_strategy = &PlacementStrategy::LeastEntropy,
                        "random" => *placement_strategy = &PlacementStrategy::Random,
                        "ordered" => *placement_strategy = &PlacementStrategy::Ordered,
                        "growing" => *placement_strategy = &PlacementStrategy::Growing,
                        _ => bail!(
                            format!(
                                "On line {} the placement_strategy can only be least_entropy, random, ordered or growing",
                                index + 1
                            )
                        )
                    }
                }
                "tile_set" => tileset_path.clone_from(&parts[1]),
                "map_size" => {
                    let max = parts[1].parse::<usize>().map_err(|_| anyhow::format_err!(
                        "Error in {} on line {}. {} is not a valid number",
                        CONFIG_FILE_PATH,
                        index + 1,
                        parts[1]
                    ))?;

                    ensure!((1..=100).contains(&max), format!(
                        "Error in {} on line {}. {} is not a accepted number. It has to be between 1 and 100",
                        CONFIG_FILE_PATH,
                        index + 1,
                        parts[1]
                    ));

                    *map_size = max;
                },
                "max_iterations" =>  {
                    let max = parts[1].parse::<u32>().map_err(|_| anyhow::format_err!(
                        "Error in {} on line {}. {} is not a valid number",
                        CONFIG_FILE_PATH,
                        index + 1,
                        parts[1]
                    ))?;

                    ensure!((100..=10000).contains(&max), format!(
                        "Error in {} on line {}. {} is not a accepted number. It has to be between 100 and 10000",
                        CONFIG_FILE_PATH,
                        index + 1,
                        parts[1]
                    ));

                    *max_iterations = max;
                },
                "seed" => {
                    let input_seed = parts[1].parse::<u64>().map_err(|_| anyhow::format_err!(
                        "Error in {} on line {}. {} is not a valid number",
                        CONFIG_FILE_PATH,
                        index + 1,
                        parts[1]
                    ))?;
                    *seed = input_seed;
                },
                _ => bail!(format!(
                    "Error in {} on line {}. {} is not a option",
                    CONFIG_FILE_PATH,
                    index + 1,
                    parts[0]
                )),
            }
        }

        // Make sure obligatory options have been set
        if tileset_path.as_str() == "" {
            bail!(format!(
                "Error in {}. Option tileset_path has not been set",
                CONFIG_FILE_PATH,
            ))
        }

        Ok(())
    }

    fn index2dto1d(&self, index: Vector2<usize>, map_size: usize) -> usize {
        index.y * map_size + index.x
    }

    fn index1dto2d(&self, index: usize, map_size: usize) -> Vector2<usize> {
        Vector2::<usize>::new(index % map_size, index / map_size)
    }

    fn within_grid(&self, index: Vector2<i32>, map_size: usize) -> bool {
        index.x >= 0 && index.y >= 0 && index.x < map_size as i32 && index.y < map_size as i32
    }

    /** PLACEMENT STRATEGIES **/
    // Chooses random tile to collapse
    fn random_placement_strategy(
        &mut self,
        tiles: &mut [Tile],
        uncollapsed_tiles: &mut BTreeSet<usize>,
        iterations: &mut u32,
        map_size: usize,
        max_iterations: u32,
        random: &mut StdRng,
    ) {
        self.collapse_center_tile(tiles, uncollapsed_tiles, random, map_size);

        while *iterations < max_iterations && !uncollapsed_tiles.is_empty() {
            let choosen_tile = *uncollapsed_tiles
                .iter()
                .choose(random)
                .expect("Set is not empty");
            uncollapsed_tiles.remove(&choosen_tile);
            self.collapse_tile(tiles, uncollapsed_tiles, choosen_tile, random, map_size);
            *iterations += 1;
        }
    }

    // Collapses tiles in the order of a BFS from the starting tile
    fn growing_placement_strategy(
        &mut self,
        tiles: &mut [Tile],
        uncollapsed_tiles: &mut BTreeSet<usize>,
        iterations: &mut u32,
        map_size: usize,
        max_iterations: u32,
        random: &mut StdRng,
    ) {
        let mut tiles_queue = VecDeque::<usize>::new();
        tiles_queue.push_back(
            self.index2dto1d(Vector2::<usize>::new(map_size / 2, map_size / 2), map_size),
        );

        while !tiles_queue.is_empty()
            && *iterations < max_iterations
            && !uncollapsed_tiles.is_empty()
        {
            let choosen_tile = tiles_queue.pop_front().expect("Queue is not empty");
            uncollapsed_tiles.remove(&choosen_tile);

            // Tile has already collapsed
            if tiles[choosen_tile].data.is_some() {
                continue; // Don't count duplicates as an iteration
            }

            self.collapse_tile(tiles, uncollapsed_tiles, choosen_tile, random, map_size);

            // Add neighbours to queue
            for direction in Direction::iterator() {
                let neighbour_position = Vector2::<i32>::new(
                    tiles[choosen_tile].tile_position.x as i32,
                    tiles[choosen_tile].tile_position.y as i32,
                ) + direction.get_vector();
                if !self.within_grid(neighbour_position, map_size) {
                    continue;
                }

                let neighbour_index = self.index2dto1d(
                    Vector2::<usize>::new(
                        neighbour_position.x as usize,
                        neighbour_position.y as usize,
                    ),
                    map_size,
                );
                let neighbour_tile: &Tile<'_> = &tiles[neighbour_index];

                // Check that tile hasn't collapsed
                if neighbour_tile.data.is_none() {
                    // Note: tile could already be in queue
                    tiles_queue.push_back(neighbour_index);
                }
            }

            *iterations += 1;
        }
    }

    // Collapese tiles in an order of left to right, down to up
    fn ordered_placement_strategy(
        &mut self,
        tiles: &mut [Tile],
        uncollapsed_tiles: &mut BTreeSet<usize>,
        iterations: &mut u32,
        map_size: usize,
        max_iterations: u32,
        random: &mut StdRng,
    ) {
        for tile_index in 0..(map_size * map_size) {
            if *iterations > max_iterations {
                return;
            }

            uncollapsed_tiles.remove(&tile_index);
            self.collapse_tile(tiles, uncollapsed_tiles, tile_index, random, map_size);
            *iterations += 1;
        }
    }

    // Collapses the tiles in the order of least entropy first
    fn least_entropy_placement_strategy(
        &mut self,
        tiles: &mut [Tile],
        uncollapsed_tiles: &mut BTreeSet<usize>,
        iterations: &mut u32,
        map_size: usize,
        max_iterations: u32,
        random: &mut StdRng,
    ) {
        self.collapse_center_tile(tiles, uncollapsed_tiles, random, map_size);

        while *iterations < max_iterations && !uncollapsed_tiles.is_empty() {
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
            self.collapse_tile(
                tiles,
                uncollapsed_tiles,
                least_entropy_index,
                random,
                map_size,
            );
            *iterations += 1;
        }
    }
}
