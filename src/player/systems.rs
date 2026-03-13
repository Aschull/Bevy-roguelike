use crate::{constants::TILE_SIZE, map::components::TileType};
use crate::player::components::*;
use bevy::prelude::*;
use bevy::input::mouse::{MouseWheel, MouseScrollUnit};
use crate::map::components::{Map, PlayerStart};
use bevy::render::mesh::{Mesh, Indices};
use crate::map::components::{FovMesh};
use crate::player::components::{Position, FacingDirection};


pub fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    map: Res<Map>, // Adicione o recurso do Mapa aqui
) {
    let tile_size = 16.0;
    let (start_x, start_y) = map.find_first_floor(); // Acha um chão real
    
    let world_x = start_x as f32 * tile_size;
    let world_y = start_y as f32 * tile_size;

    commands.spawn((
        Player,
        Position { 
            x: start_x, 
            y: start_y, 
            x_float: world_x, 
            y_float: world_y 
        },
        FacingDirection(0.0),
        SpriteBundle {
            texture: asset_server.load("player.png"),
            sprite: Sprite {
                custom_size: Some(Vec2::new(32.0, 32.0)),
                ..default()
            },
            transform: Transform::from_xyz(world_x, world_y, 2.0),
            ..default()
        },
    ));
}


pub fn handle_input(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    map: Res<Map>,
    mut query: Query<(&mut Position, &mut Transform, &mut FacingDirection), With<Player>>,
) {
    let Ok((mut pos, mut transform, mut facing)) = query.get_single_mut() else { return };

    let speed = 150.0;
    let tile_size = 16.0;
    let mut velocity = Vec2::ZERO;

    if keys.pressed(KeyCode::KeyW) { velocity.y += 1.0; }
    if keys.pressed(KeyCode::KeyS) { velocity.y -= 1.0; }
    if keys.pressed(KeyCode::KeyA) { velocity.x -= 1.0; }
    if keys.pressed(KeyCode::KeyD) { velocity.x += 1.0; }

    if velocity != Vec2::ZERO {
        let direction = velocity.normalize();
        let move_delta = direction * speed * time.delta_seconds();
        
        // Nova posição potencial
        let next_x = pos.x_float + move_delta.x;
        let next_y = pos.y_float + move_delta.y;

        // Checagem de colisão simples (verifica o tile no centro do player)
        let grid_x = (next_x / tile_size).round() as i32;
        let grid_y = (next_y / tile_size).round() as i32;

        if grid_x >= 0 && grid_x < map.width && grid_y >= 0 && grid_y < map.height {
            let idx = (grid_y * map.width + grid_x) as usize;
            
            // Só move se não for parede
            if map.tiles[idx] == TileType::Floor {
                pos.x_float = next_x;
                pos.y_float = next_y;
                
                pos.x = grid_x;
                pos.y = grid_y;

                transform.translation.x = pos.x_float;
                transform.translation.y = pos.y_float;
            }
        }
        
        facing.0 = direction.y.atan2(direction.x);
    }
}

pub fn update_translation(mut query: Query<(&Position, &mut Transform), Changed<Position>>) {
    for (pos, mut transform) in &mut query {
        transform.translation.x = pos.x as f32 * TILE_SIZE;
        transform.translation.y = pos.y as f32 * TILE_SIZE;
    }
}

pub fn camera_follow(
    player_query: Query<&Transform, (With<Player>, Without<Camera>)>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    time: Res<Time>,
) {
    let Ok(player_transform) = player_query.get_single() else { return };
    
    // CORREÇÃO AQUI: Adicionado o "if" antes do "let"
    if let Ok(mut camera_transform) = camera_query.get_single_mut() {
        let target = player_transform.translation;
        let smoothness = 5.0;

        let new_pos = camera_transform.translation.lerp(target, time.delta_seconds() * smoothness);
        
        camera_transform.translation.x = new_pos.x;
        camera_transform.translation.y = new_pos.y;
    }
}

pub fn camera_zoom(
    mut query: Query<&mut OrthographicProjection, With<Camera2d>>,
    mut scroll_evr: EventReader<MouseWheel>,
) {
    for event in scroll_evr.read() {
        if let Ok(mut projection) = query.get_single_mut() {
            // No Bevy, quanto MENOR a escala, mais "perto" (Zoom In)
            // Quanto MAIOR a escala, mais "longe" (Zoom Out)
            let factor = match event.unit {
                MouseScrollUnit::Line => 1.0 - event.y * 0.1,
                MouseScrollUnit::Pixel => 1.0 - event.y * 0.005,
            };

            projection.scale *= factor;

            // Clamp: evita que o zoom fique negativo ou infinitamente longe
            projection.scale = projection.scale.clamp(0.2, 5.0);
        }
    }
}

pub fn update_fov(mut map: ResMut<Map>, player_query: Query<&Position, With<Player>>) {
    let Ok(pos) = player_query.get_single() else { return };

    // Limpa visibilidade anterior
    for v in map.visible_tiles.iter_mut() { *v = false; }

    let range = 15; // Range em tiles de 16px
    for y in (pos.y - range)..=(pos.y + range) {
        for x in (pos.x - range)..=(pos.x + range) {
            if x < 0 || x >= map.width || y < 0 || y >= map.height { continue; }
            
            // Verificação de distância simples (círculo)
            let dist = ((x - pos.x).pow(2) + (y - pos.y).pow(2)) as f32;
            if dist < (range as f32).powi(2) {
                let idx = (y * map.width + x) as usize;
                map.visible_tiles[idx] = true;
                map.explored_tiles[idx] = true;
            }
        }
    }
}

pub fn update_fov_geometry(
    player_query: Query<(&Transform, &FacingDirection), With<Player>>,
    map: Res<Map>,
    fov_mesh_query: Query<&Handle<Mesh>, With<FovMesh>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let Ok((player_t, facing)) = player_query.get_single() else { return };
    let Ok(mesh_handle) = fov_mesh_query.get_single() else { return };
    let Some(mesh) = meshes.get_mut(mesh_handle) else { return };

    let tile_size = 16.0;
    let range_pixels = 300.0; 
    let num_rays = 200; 
    let cone_angle = 125.0f32.to_radians();

    let player_center = player_t.translation.truncate();
    let (sin_f, cos_f) = facing.0.sin_cos();
    
    // Aumentamos o offset para 20.0 para a luz sair claramente de fora do corpo do player
    let origin_offset = Vec2::new(cos_f * 20.0, sin_f * 20.0);
    let ray_origin = player_center + origin_offset;

    let mut vertices = Vec::new();
    let mut colors = Vec::new();
    
    // Centro da luz (Origem)
    vertices.push(Vec3::new(origin_offset.x, origin_offset.y, 0.0));
    colors.push([2.0, 2.0, 1.2, 1.0]); 

    for i in 0..=num_rays {
        let angle = facing.0 - (cone_angle / 2.0) + (i as f32 * (cone_angle / num_rays as f32));
        let ray_dir = Vec2::new(angle.cos(), angle.sin());
        let mut hit_dist = range_pixels;

        // Raycast preciso: ignora os primeiros 2 pixels para não colidir com o próprio player
        for d in (2..range_pixels as i32).step_by(2) {
            let check_pt = ray_origin + ray_dir * d as f32;
            let gx = (check_pt.x / tile_size).floor() as i32;
            let gy = (check_pt.y / tile_size).floor() as i32;

            if gx < 0 || gx >= map.width || gy < 0 || gy >= map.height {
                hit_dist = d as f32;
                break;
            }
            
            if map.tiles[(gy * map.width + gx) as usize] == TileType::Wall {
                hit_dist = d as f32;
                break;
            }
        }

        let world_hit = ray_origin + ray_dir * hit_dist;
        let local_hit = world_hit - player_center;
        
        vertices.push(Vec3::new(local_hit.x, local_hit.y, 0.0));
        
        // Gradiente de queda de luz
        let falloff = (1.0 - (hit_dist / range_pixels)).powf(2.0);
        colors.push([1.5, 1.5, 1.0, falloff * 0.7]);
    }

    let mut indices = Vec::new();
    for i in 1..num_rays as u32 {
        indices.push(0);
        indices.push(i);
        indices.push(i + 1);
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));
}


pub fn sync_fov_mesh(
    // Pegamos o transform do player (Without evita conflito de query)
    player_query: Query<&Transform, (With<Player>, Without<FovMesh>)>,
    mut mesh_query: Query<&mut Transform, With<FovMesh>>,
) {
    if let Ok(player_t) = player_query.get_single() {
        if let Ok(mut mesh_t) = mesh_query.get_single_mut() {
            // A malha "cola" no player. 
            // Z=1.0 (Luz) fica ABAIXO do Player (Z=2.0) e ACIMA do chão (Z=0.0)
            mesh_t.translation.x = player_t.translation.x;
            mesh_t.translation.y = player_t.translation.y;
            mesh_t.translation.z = 1.0; 
        }
    }
}


pub fn generate_map(width: i32, height: i32) -> Map {
    // Começamos com tudo preenchido de parede
    let mut tiles = vec![TileType::Wall; (width * height) as usize];
    
    // Definimos o tamanho do "pincel" para escavar (2 para cada lado = 5 tiles total)
    let brush_radius = 2; 

    // Exemplo de escavação de uma sala grande central
    for y in 10..(height - 10) {
        for x in 10..(width - 10) {
            let idx = (y * width + x) as usize;
            tiles[idx] = TileType::Floor;
        }
    }

    // Exemplo de corredor horizontal LARGO
    let mid_y = height / 2;
    for x in 5..(width - 5) {
        for dy in -brush_radius..=brush_radius {
            let ty = mid_y + dy;
            if ty > 0 && ty < height {
                let idx = (ty * width + x) as usize;
                tiles[idx] = TileType::Floor;
            }
        }
    }

    // Exemplo de corredor vertical LARGO
    let mid_x = width / 2;
    for y in 5..(height - 5) {
        for dx in -brush_radius..=brush_radius {
            let tx = mid_x + dx;
            if tx > 0 && tx < width {
                let idx = (y * width + tx) as usize;
                tiles[idx] = TileType::Floor;
            }
        }
    }

    Map {
        width,
        height,
        tiles,
        explored_tiles: vec![false; (width * height) as usize],
        visible_tiles: vec![false; (width * height) as usize],
    }
}
