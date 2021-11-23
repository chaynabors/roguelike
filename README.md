# roguelike
A game about exploring with friends... with graphics not *too* far from DOS.


## Todo
* Domain
    - [x] Type definitions
    - [x] Type de/serialization
* Rendering
    - [x] Device interaction
    - [x] Screen Clearing
    - [x] Tile rendering
    - [x] Simple distance lighting
    - [ ] Colored lights
    - [ ] Arbitrarily numbered lights
    - [ ] Chained tile rendering
    - [ ] Light dropoff on wall penetration
* Audio
    - [ ] Simple audio playback
* Networking
    - [ ] Messaging and handshake protocol
    - [ ] Version comparison
    - [ ] P2P state manipulation
* Content
    - [ ] Procedural map generation
    - [ ] Primitive set of sounds
    ~~- [ ] Character set (font)~~
    - [x] 2 color spritesheet with alpha
* Gameplay
    - [ ] Collision detection/resolution
    - [ ] Character controller
    - [ ] Character customization


## Type Definitions
Map:
* layout: Vec<Material>
* lights: Vec<Light>

Material:
* Enum

Light:
* position: Vector2
* color: Color
* luminosity: float

Vector2:
* x: float
* y: float

Color:
* r: float
* g: float
* b: float

Camera:
* position: Vector2

Player:
* position: Vector2

State:
* map: Map
* camera: Camera
* player: Player
