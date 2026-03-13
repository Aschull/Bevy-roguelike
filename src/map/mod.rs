pub mod components;
pub mod systems;

use bevy::prelude::*;

use crate::map::components::{FovMesh, Map, PlayerStart, TileType};
use crate::player::systems as player_systems; // Dá o apelido para o plugin encontrar
use crate::map::systems as map_systems;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle}; 
use bevy::render::render_asset::RenderAssetUsages; // O caminho que o compilador sugeriu
use bevy::render::mesh::{Mesh, Indices};
use bevy::render::render_resource::PrimitiveTopology;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        let (map, x, y) = Map::new_labyrinth(300, 350);

        app.insert_resource(map)
            .insert_resource(PlayerStart(x, y))
            .add_systems(Startup, setup_map_assets) 
            .add_systems(
                Update,
                (
                    // Lógica de Visão e Renderização em sequência
                    (
                        player_systems::update_fov, 
                        // map_systems::render_fov,
                        map_systems::render_world_textured,
                    ).chain()
                )
                // ESSA É A CHAVE: Só roda após o input do player ser processado
                .after(player_systems::handle_input), 
            );
    }
    
}
pub fn setup_fov_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut mesh = Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::MAIN_WORLD | bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
    );

    commands.spawn((
        FovMesh,
        MaterialMesh2dBundle {
            mesh: meshes.add(mesh).into(),
            material: materials.add(ColorMaterial {
                // BRANCO PURO e FORTE. O valor 2.0 faz ele brilhar.
                color: Color::rgba(2.0, 2.0, 2.0, 1.0), 
                ..default()
            }),
            // Z = 5.0 (Garante que está MUITO acima do chão que é 0.0)
            transform: Transform::from_xyz(0.0, 0.0, 5.0),
            ..default()
        },
    ));
}

// O "Setup" propriamente dito: aqui carregamos as imagens
fn setup_map_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(MapAssets {
        wall_texture: asset_server.load("wall-0001.png"),
        floor_texture: asset_server.load("floor-0001.png"),
    });
}

#[derive(Resource)]
pub struct MapAssets {
    pub wall_texture: Handle<Image>,
    pub floor_texture: Handle<Image>,
}

impl Map {
    pub fn new(width_32: i32, height_32: i32) -> Self {
        // Dobramos a dimensão para caber 4 tiles (16px) no lugar de 1 (32px)
        let width = width_32 * 2;
        let height = height_32 * 2;
        
        Self {
            width,
            height,
            tiles: vec![TileType::Floor; (width * height) as usize],
            explored_tiles: vec![false; (width * height) as usize],
            visible_tiles: vec![false; (width * height) as usize],
        }
    }
}

impl Map {
    pub fn find_first_floor(&self) -> (i32, i32) {
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = (y * self.width + x) as usize;
                if self.tiles[idx] == TileType::Floor {
                    return (x, y);
                }
            }
        }
        (10, 10) // Fallback caso não ache nada
    }
}