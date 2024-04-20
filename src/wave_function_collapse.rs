use std::{
    f32::consts::PI,
    fs::File,
    io::{BufRead, BufReader},
};

use anyhow::{bail, ensure};
use nalgebra::{Rotation3, Vector2};

use crate::{model_loader::ModelLoader, scene::Scene, tile::Tile, tile_data::TileData};

pub struct WFC<'a> {
    map_size: usize,
    scene: &'a mut Scene,
}

impl<'a> WFC<'a> {
    pub fn new(scene: &'a mut Scene, map_size: usize) -> Self {
        WFC { map_size, scene }
    }

    // Where the actual Wave Function Collapse logic happens
    pub fn place_tiles(&mut self) -> anyhow::Result<()> {
        // TODO: continue until filled map
        match self.load_tiles() {
            Ok(tile_datas) => {
                let possible_tiles: Vec<&TileData> = tile_datas.iter().collect();

                // Fill tiles list with all possibilities
                let mut tiles: Vec<Tile> = vec![];
                for i in 0..(self.map_size * self.map_size) {
                    tiles.push(Tile::new(possible_tiles.clone(), self.index1dto2d(i)));
                }

                // Collapse first tile
                tiles[0].collapse(self.scene);

                Ok(())
            }
            Err(err) => Err(err),
        }
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
                                values.len() == 6,
                                format!(
                                    "Line {} \"{}\" does not contain all values required",
                                    index + 1,
                                    line
                                )
                            );
                            ensure!(
                                values[5] == "true" || values[5] == "false",
                                format!(
                                    "On line {} the rotatable value can only be true or false",
                                    index + 1
                                )
                            );

                            // Load models
                            let rotation_angles: Vec<Rotation3<f32>> = match values[5].as_str() {
                                "true" => vec![
                                    Rotation3::from_euler_angles(0.0, 0.0, 0.0),
                                    Rotation3::from_euler_angles(0.0, 0.0, PI / 2.0),
                                    Rotation3::from_euler_angles(0.0, 0.0, PI),
                                    Rotation3::from_euler_angles(0.0, 0.0, 3.0 / 2.0 * PI),
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
                                            weight: 1, // TODO: expose to config
                                            up_edge: values[1 + (index % 4)].clone(),
                                            right_edge: values[1 + ((index + 1) % 4)].clone(),
                                            down_edge: values[1 + ((index + 2) % 4)].clone(),
                                            left_edge: values[1 + ((index + 3) % 4)].clone(),
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
}
