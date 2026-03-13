use bevy::prelude::*;
use crate::map::components::{Map, MapOverlay, TileType};
use crate::player::components::{Player, Position};

pub fn toggle_map(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut map_overlay: ResMut<MapOverlay>,
) {
    if keyboard.just_pressed(KeyCode::KeyM) {
        map_overlay.is_open = !map_overlay.is_open;
        println!("Mapa aberto: {}", map_overlay.is_open); // Debug básico
    }
}

pub fn draw_map_overlay(
    map: Res<Map>,
    map_overlay: Res<MapOverlay>,
    player_query: Query<&Position, With<Player>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut gizmos: Gizmos,
) {
    // Se o mapa não estiver aberto, não processa nada
    if !map_overlay.is_open { return; }

    // 1. Pegamos a posição da câmera para desenhar o mapa "seguindo" a tela
    let (camera, camera_transform) = camera_query.single();
    let Some(window_size) = camera.logical_viewport_size() else { return };
    
    // Calculamos o centro da tela no mundo
    let center = camera_transform.translation().truncate();

    // 2. Configurações de escala do mapa (ajuste o scale para caber melhor)
    let scale = 10.0; 
    let map_width_scaled = map.width as f32 * scale;
    let map_height_scaled = map.height as f32 * scale;
    
    // Offset para centralizar o desenho
    let offset = center - Vec2::new(map_width_scaled / 2.0, map_height_scaled / 2.0);

    // 3. Fundo do Mapa (Semi-transparente para dar contraste)
    gizmos.rect_2d(center, 0.0, Vec2::new(map_width_scaled + 20.0, map_height_scaled + 20.0), Color::rgba(0.0, 0.0, 0.0, 0.85));

    // 4. Desenho Otimizado (Step de 2 para não congelar)
    for y in (0..map.height).step_by(2) {
        for x in (0..map.width).step_by(2) {
            let idx = (y * map.width + x) as usize;

            if map.explored_tiles[idx] {
                let color = if map.tiles[idx] == TileType::Wall {
                    Color::rgb(0.4, 0.4, 0.4) // Paredes descobertas
                } else {
                    Color::rgb(0.15, 0.15, 0.2) // Chão descoberto
                };

                let pos = offset + Vec2::new(x as f32 * scale, y as f32 * scale);
                gizmos.rect_2d(pos, 0.0, Vec2::splat(scale * 2.0), color);
            }
        }
    }

    // 5. Indicador do Jogador (Ponto piscante no mapa)
    if let Ok(p_pos) = player_query.get_single() {
        let p_map_pos = offset + Vec2::new(p_pos.x as f32 * scale, p_pos.y as f32 * scale);
        gizmos.rect_2d(p_map_pos, 0.0, Vec2::splat(scale * 4.0), Color::GREEN);
    }
}