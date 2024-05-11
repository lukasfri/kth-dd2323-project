# Wave function collapse (WFC)
This is a project that implements wave function collapse using Rust and Winit. The renderer used is a rasterized renderer which offloads computation to the GPU. Using WFC this program stiches together 3D tiles to create a coherent 3D world. Some examples that it can generate (using green_city and oasis tilesets):

[ADD EXAMPLE IMAGES]

This project focuses on usability by letting the user easily change generation parameters by modifying config files. This also allows the user to easily upload their own tileset for use in the program. See [documentation](#documentation-configtxt).

Run the program by running the following command in the terminal:

```
WAYLAND_DISPLAY="" cargo run
```

You can move the camera using W/S/A/D/Q/E. You can rotate the camera by right clicking and dragging or using the Left/Right/Up/Down keys.

## Explanation WFC
Wave function collapsed works by first assigning each tile position every possible tile, each tile position has a so called wave function with multiple possible outputs. This is the start state for the algorithm. It then chooses a tile position to collapse, it is assigned one of its possible tiles. This then affects its neighbouring tiles so their wave function is updated. This process continues until the entire map has been filled, the max iteration count has been reached or the map has reached an impossible state where no more tile positions can be collapsed. It chooses the order of tile position collapse depending on the placement strategy used. Our project implements the following strategies:

Least entropy (default): collapse the tile position with the least options

Random: chooses a random tile position to collapse

Ordered: collapses tile positions from the top left corner. Goes left-to-right, up-to-down

Growing: collapses tile positions out from the last collapsed tile position according to a BFS (breadth first search)

In our testing we could conclude that the least entropy strategy works the best as it assigns tiles to tile positions that have few possible options. This mostly prevents tile positions from not having any possible tiles to collapse to.

## Documentation config.txt

This is the main config file. Each row configures one parameter in the format of PARAMETER=VALUE. The available parameters are:

|Parameter  |Possible values|Information|Default|
|-----------|---------------|-----------|--|
|tile_set   |A path to a tileset folder |The folder is relative to the project root. The folder must contain a tiles_config.txt file for configuring the tileset. This setting is **required**|-|
|placement_strategy|"least_entropy", "random", "ordered" or "growing"| Chooses the order that the algorithm collapses tiles.| "least_entropy"|
|map_size|An integer between 1 and 100|Defines the width and height of the generated math|10|
|max_iterations|An integer between 100 and 10000|Defines the maxinum amount of iterations the algorithm does before stopping. It also stops when the map is filled or no tiles can be collapsed anymore|1000|
|seed|Non negative 64 bit integer|Seed is used for everything random. By setting a seed you get a deterministic output. Setting the seed to 0 randomises the seed|0|

An example of a valid config file:
```
placement_strategy=least_entropy
tile_set=./assets/green_city
map_size=10
max_iterations=500
seed=100
```

## Documentation tiles_config.txt

This file should contain a row for each tile included in the tileset. Parameters are seperated by comma and have to be in the order shown below. All parameters need to be set.

|Parameter  |Possible values|Information|
|-----------|---------------|-----------|
|model|A file path relative to the tileset folder|Defines the path to the .glb model file|
|weight|A positive 32-bit integer|Defines the priority for selecting this tile over others. Note that this is per tile variant, if there are 4 rotations of the tile each variant has this priority.|
|up-edge|A string|Defines the upwards type of the tile. This is used to determine which tiles can be placed above this tile. If this tile has grass as up-edge for example then the tile above needs to have grass as down-edge.|
|right-edge|A string|Defines the right type of the tile. This is used to determine which tiles can be placed to the right of this tile.|
|down-edge|A string|Defines the downwards type of the tile. This is used to determine which tiles can be placed below this tile.|
|left-edge|A string|Defines the left type of the tile. This is used to determine which tiles can be placed to the left of this tile.|
|rotations|"1", "2" or "4"|Defines how many rotation variants should be generated for the tile. 1 does not create any new variants. 2 creates one new variant that is rotated 90 degress clockwise from the original tile. 4 generates all 4 rotations.

A valid example of tiles config:
```
grass.glb, 16, grass, grass, grass, grass, 1
pond0.glb, 1, grass, grass, grass, grass, 1
pond1.glb, 1, pond, grass, grass, grass, 4
pond2.glb, 1, pond, grass, pond, grass, 2
```