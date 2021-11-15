# roguelike
 
## Maps
Maps are the levels that make up the game world.

### Definitions
- [ ] Map:
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

- [ ] State:
* map: Map
* camera: Camera
* player: Player
