use bevy::prelude::*;
use bevy::render::mesh::Mesh;
use rand::prelude::*;

#[derive(Copy, Clone, PartialEq, Debug)] 
pub enum TileType {
    Wall,
    Floor,
}

// Adicionamos Resource para o Bevy permitir o uso de Res<Map>
#[derive(Resource)] 
pub struct Map {
    pub tiles: Vec<TileType>,
    pub width: i32,
    pub height: i32,
    pub visible_tiles: Vec<bool>,
    pub explored_tiles: Vec<bool>, // Memória do mapa (acumulativo)
}

impl Map {
    // Retorna true se a coordenada estiver dentro do mapa e NÃO for uma parede
    pub fn is_passable(&self, x: i32, y: i32) -> bool {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return false; // Fora do mapa não é passável
        }
        let idx = (y * self.width + x) as usize;
        self.tiles[idx] != TileType::Wall // Passável se NÃO for parede
    }



    /// Versão Nova: Labirinto de Cavernas (Drunkard's Walk)
    pub fn new_cave(width: i32, height: i32) -> Self {
        // Começamos com o mapa TODO preenchido de paredes
        let mut tiles = vec![TileType::Wall; (width * height) as usize];
        let mut rng = rand::rng();

        // Ponto inicial do "escavador"
        let mut x = 5;
        let mut y = 5;
        tiles[(y * width + x) as usize] = TileType::Floor;

        // O "Bêbado" vai dar 40.000 passos cavando o chão
        for _ in 0..20000 {
            let direction = rng.random_range(0..4);
            match direction {
                0 => y -= 1, // Norte
                1 => y += 1, // Sul
                2 => x -= 1, // Oeste
                3 => x += 1, // Leste
                _ => {}
            }

            // Mantém dentro das bordas
            x = x.clamp(1, width - 2);
            y = y.clamp(1, height - 2);

            let idx = (y * width + x) as usize;
            tiles[idx] = TileType::Floor;
        }

        Self::init_struct(tiles, width, height)
    }

    pub fn new_labyrinth(width: i32, height: i32) -> (Self, i32, i32) {
        let mut tiles = vec![TileType::Wall; (width * height) as usize];
        let mut rooms: Vec<Rect> = Vec::new();
        let mut rng = rand::rng();

        const MAX_ROOMS: i32 = 40;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 12;

        for _ in 0..MAX_ROOMS {
            let w = rng.random_range(MIN_SIZE..MAX_SIZE);
            let h = rng.random_range(MIN_SIZE..MAX_SIZE);
            let x = rng.random_range(1..width - w - 1);
            let y = rng.random_range(1..height - h - 1);

            let new_room = Rect::new(x, y, w, h);
            
            let mut ok = true;
            for other_room in rooms.iter() {
                if new_room.intersect(other_room) {
                    ok = false;
                    break;
                }
            }

            if ok {
                Self::apply_room_to_map(&mut tiles, &new_room, width);

                if !rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = rooms[rooms.len() - 1].center();

                    if rng.random_bool(0.5) {
                        Self::apply_horizontal_tunnel(&mut tiles, prev_x, new_x, prev_y, width);
                        Self::apply_vertical_tunnel(&mut tiles, prev_y, new_y, new_x, width);
                    } else {
                        Self::apply_vertical_tunnel(&mut tiles, prev_y, new_y, prev_x, width);
                        Self::apply_horizontal_tunnel(&mut tiles, prev_x, new_x, new_y, width);
                    }
                }
                rooms.push(new_room);
            }
        }

        // A MÁGICA DO SPAWN:
        // Pegamos o centro da primeira sala para o player não nascer na parede
        let (spawn_x, spawn_y) = if !rooms.is_empty() {
            rooms[0].center()
        } else {
            (width / 2, height / 2) // Fallback caso não consiga criar salas
        };

        (Self::init_struct(tiles, width, height), spawn_x, spawn_y)
    }

    fn apply_room_to_map(tiles: &mut [TileType], room: &Rect, map_width: i32) {
        for y in room.y1..room.y2 {
            for x in room.x1..room.x2 {
                let idx = (y * map_width + x) as usize;
                tiles[idx] = TileType::Floor;
            }
        }
    }

    fn apply_horizontal_tunnel(tiles: &mut [TileType], x1: i32, x2: i32, y: i32, width: i32) {
        use std::cmp::{min, max};
        for x in min(x1, x2)..=max(x1, x2) {
            let idx = (y * width + x) as usize;
            tiles[idx] = TileType::Floor;
        }
    }

    fn apply_vertical_tunnel(tiles: &mut [TileType], y1: i32, y2: i32, x: i32, width: i32) {
        use std::cmp::{min, max};
        for y in min(y1, y2)..=max(y1, y2) {
            let idx = (y * width + x) as usize;
            tiles[idx] = TileType::Floor;
        }
    }

    // Função auxiliar para evitar repetição de código
    fn init_struct(tiles: Vec<TileType>, width: i32, height: i32) -> Self {
        let tiles_count = (width * height) as usize;
        Self {
            tiles,
            visible_tiles: vec![false; tiles_count],
            explored_tiles: vec![false; tiles_count],
            width,
            height,
        }
    }
}

#[derive(Component)]
pub struct WallMarker;

#[derive(Resource)]
pub struct PlayerStart(pub i32, pub i32);


struct Rect {
    x1: i32, x2: i32, y1: i32, y2: i32,
}

impl Rect {
    fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Rect { x1: x, y1: y, x2: x + w, y2: y + h }
    }
    fn intersect(&self, other: &Rect) -> bool {
        self.x1 <= other.x2 && self.x2 >= other.x1 && self.y1 <= other.y2 && self.y2 >= other.y1
    }
    fn center(&self) -> (i32, i32) {
        ((self.x1 + self.x2) / 2, (self.y1 + self.y2) / 2)
    }
}


#[derive(Resource, Default)]
pub struct MapOverlay {
    pub is_open: bool,
}

#[derive(Component)]
pub struct FovMesh;