use crate::{constants::TILE_SIZE, map::components::TileType};
use crate::player::components::*;
use bevy::prelude::*;
use super::MovementTimer;
use bevy::input::mouse::{MouseWheel, MouseScrollUnit};
use crate::map::components::{Map, PlayerStart};

pub fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    map: Res<Map>, // Injetamos o mapa aqui
) {
    // 1. Encontramos a posição segura
    let (safe_x, safe_y) = map.find_first_floor();
    
    let tile_size = 32.0;

    commands.spawn((
        Player,
        Position { x: safe_x, y: safe_y },
        SpriteBundle {
            texture: asset_server.load("player.png"),
            sprite: Sprite {
                custom_size: Some(Vec2::splat(tile_size)),
                ..default()
            },
            // 2. Colocamos o transform na posição segura calculada
            transform: Transform::from_xyz(
                safe_x as f32 * tile_size,
                safe_y as f32 * tile_size,
                1.0, // Z-index para ficar acima do chão
            ),
            ..default()
        },
        FacingDirection(0.0),
    ));
}


pub fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut timer: ResMut<MovementTimer>,
    // Usamos Query de forma segura
    mut query: Query<(&mut Position, &mut FacingDirection), With<Player>>,
    map: Res<crate::map::components::Map>,
    map_overlay: Res<crate::map::components::MapOverlay>,
) {
    // 1. Se o mapa estiver aberto, não processamos nada
    if map_overlay.is_open { return; }

    // 2. Atualizamos o timer de movimento
    timer.0.tick(time.delta());

    // 3. Só tentamos mover se o timer terminou E o jogador existe
    if timer.0.just_finished() {
        // Substituímos o .single_mut() por if let para evitar Panics
        if let Ok((mut pos, mut facing)) = query.get_single_mut() {
            let mut dx = 0;
            let mut dy = 0;

            // Mapeamento de teclas
            if keyboard.pressed(KeyCode::KeyW) { dy += 1; }
            if keyboard.pressed(KeyCode::KeyS) { dy -= 1; }
            if keyboard.pressed(KeyCode::KeyA) { dx -= 1; }
            if keyboard.pressed(KeyCode::KeyD) { dx += 1; }

            // Só processamos se houver intenção de movimento
            if dx != 0 || dy != 0 {
                // --- ATUALIZAÇÃO DA DIREÇÃO (Ângulo do Cone) ---
                // atan2(y, x) retorna o ângulo correto para onde o jogador está "olhando"
                facing.0 = (dy as f32).atan2(dx as f32);

                // --- LÓGICA DE COLISÃO ---
                let new_x = pos.x + dx;
                let new_y = pos.y + dy;

                if map.is_passable(new_x, new_y) {
                    pos.x = new_x;
                    pos.y = new_y;
                }
            }
        }
    }
}

pub fn update_translation(mut query: Query<(&Position, &mut Transform), Changed<Position>>) {
    for (pos, mut transform) in &mut query {
        transform.translation.x = pos.x as f32 * TILE_SIZE;
        transform.translation.y = pos.y as f32 * TILE_SIZE;
    }
}

pub fn camera_follow(
    player_query: Query<&Position, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
) {
    // 1. Pegamos a posição lógica do player
    if let Ok(player_pos) = player_query.get_single() {
        // 2. Pegamos o transform da câmera
        if let Ok(mut camera_transform) = camera_query.get_single_mut() {
            let tile_size = 32.0;
            
            // 3. Suavização (Opcional): 
            // Você pode só igualar ou usar um lerp para uma câmera mais "elástica"
            let target_x = player_pos.x as f32 * tile_size;
            let target_y = player_pos.y as f32 * tile_size;

            camera_transform.translation.x = target_x;
            camera_transform.translation.y = target_y;
        }
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

pub fn update_fov(
    player_query: Query<(&Position, &FacingDirection), With<Player>>,
    mut map: ResMut<Map>,
) {
    if let Ok((pos, facing)) = player_query.get_single() {
        // 1. Resetamos a visibilidade do frame anterior
        map.visible_tiles.fill(false);

        let range = 15.0; // Alcance da luz
        let cone_angle = 145.0f32.to_radians();
        let half_cone = cone_angle / 2.0;
        let num_rays = 180; // Quantidade de raios para cobrir o cone sem buracos

        // --- CÁLCULO DO CONE DE VISÃO ---
        for i in 0..num_rays {
            // Ângulo do raio atual dentro do arco de 145 graus
            let ray_angle = facing.0 - half_cone + (i as f32 * (cone_angle / num_rays as f32));
            
            let cos = ray_angle.cos();
            let sin = ray_angle.sin();

            for step in 0..range as i32 {
                let x = pos.x + (cos * step as f32).round() as i32;
                let y = pos.y + (sin * step as f32).round() as i32;

                // Verifica limites do mapa
                if x >= 0 && x < map.width && y >= 0 && y < map.height {
                    let idx = (y * map.width + x) as usize;
                    
                    // Marcamos como visível e explorado ANTES do check de parede
                    map.visible_tiles[idx] = true;
                    map.explored_tiles[idx] = true;

                    // LÓGICA ANTI-X-RAY:
                    // Se o tile atual for uma parede, nós a iluminamos (para o player ver o '#' ),
                    // mas interrompemos o raio (break) para que ele não revele o que está atrás.
                    if map.tiles[idx] == TileType::Wall {
                        break; 
                    }
                } else {
                    // Saiu dos limites do mapa, para o raio atual
                    break;
                }
            }
        }

        // --- VISÃO PERIFÉRICA (360º de curto alcance) ---
        // Simula a percepção imediata ao redor do corpo (evita ficar cego ao girar)
        for i in 0..36 {
            let angle = (i as f32 * 10.0f32).to_radians();
            for step in 1..3 { // Raio de 2 tiles
                let x = pos.x + (angle.cos() * step as f32).round() as i32;
                let y = pos.y + (angle.sin() * step as f32).round() as i32;

                if x >= 0 && x < map.width && y >= 0 && y < map.height {
                    let idx = (y * map.width + x) as usize;
                    map.visible_tiles[idx] = true;
                    map.explored_tiles[idx] = true;
                    
                    if map.tiles[idx] == TileType::Wall { break; }
                }
            }
        }
    }
}