# roguelike
A game about exploring with friends... in the terminal.

### Todo
* Domain
    - [x] Type definitions
    - [x] Type de/serialization
* Rendering
    - [x] Device interaction
    - [x] Screen Clearing
    - [ ] Texture blitting
    - [ ] Render pass per light
* Audio
    - [ ] Simple audio playback
* Networking
    - [ ] Messaging and handshake protocol
    - [ ] Version comparison
    - [ ] P2P state manipulation
* Content
    - [ ] Procedural map generation
    - [ ] Primitive set of sounds
    - [ ] Character set (font)
* Gameplay
    - [ ] Collision detection/resolution
    - [ ] Character controller

### Type Definitions
- [x] Map:
* layout: Vec<Material>
* lights: Vec<Light>

- [x] Material: Enum

- [x] Light:
* position: Vector2
* color: Color
* luminosity: float

- [x] Vector2:
* x: float
* y: float

- [x] Color:
* r: float
* g: float
* b: float

- [x] Camera:
* position: Vector2

- [x] Player:
* position: Vector2

- [x] State:
* map: Map
* camera: Camera
* player: Player
