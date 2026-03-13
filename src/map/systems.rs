use bevy::prelude::*;
use crate::{map::components::WallMarker, player::components::{Player, Position}};

use super::components::{Map, TileType};


pub fn render_world_fast(
    map: Res<Map>,
    player_query: Query<&Position, With<Player>>,
    mut gizmos: Gizmos,
) {
    let Ok(player_pos) = player_query.get_single() else { return };

    // 1. OTIMIZAÇÃO: Distância de Renderização (Culling)
    // Em vez de percorrer 100.000 tiles, vamos percorrer apenas ~900 (30x30)
    let render_distance = 15; 
    let tile_size = 32.0;

    // Calculamos os limites do loop baseados na posição do jogador
    let start_x = (player_pos.x - render_distance).max(0);
    let end_x = (player_pos.x + render_distance).min(map.width - 1);
    let start_y = (player_pos.y - render_distance).max(0);
    let end_y = (player_pos.y + render_distance).min(map.height - 1);

    for y in start_y..=end_y {
        for x in start_x..=end_x {
            let idx = (y * map.width + x) as usize;

            // 2. FOG OF WAR: Só desenha se o jogador já "viu" esse tile antes
            if map.explored_tiles[idx] {
                let pos = Vec2::new(x as f32 * tile_size, y as f32 * tile_size);
                
                // 3. CORES DINÂMICAS: Diferencia o que você vê AGORA do que é MEMÓRIA
                let color = if map.visible_tiles[idx] {
                    if map.tiles[idx] == TileType::Wall {
                        Color::rgb(0.6, 0.6, 0.6) // Parede visível (clara)
                    } else {
                        Color::rgb(0.2, 0.2, 0.4) // Chão visível (azuladinho)
                    }
                } else {
                    if map.tiles[idx] == TileType::Wall {
                        Color::rgb(0.1, 0.1, 0.1) // Parede explorada (escura)
                    } else {
                        Color::rgb(0.02, 0.02, 0.05) // Chão explorado (quase preto)
                    }
                };

                // Desenha o tile
                gizmos.rect_2d(
                    pos, 
                    0.0, 
                    Vec2::splat(tile_size - 1.0), 
                    color
                );
            }
        }
    }
}


pub fn setup_minimap(mut commands: Commands) {
    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Px(150.0),  // Tamanho fixo do minimapa
            height: Val::Px(150.0),
            position_type: PositionType::Absolute,
            right: Val::Px(20.0),   // Distância da borda direita
            top: Val::Px(20.0),     // Distância do topo
            border: UiRect::all(Val::Px(2.0)),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        background_color: Color::rgba(0.0, 0.0, 0.0, 0.8).into(), // Fundo semi-transparente
        border_color: Color::WHITE.into(),
        ..default()
    }).insert(Name::new("MinimapContainer"));
}

pub fn render_fov(
    map: Res<Map>,
    mut query: Query<(&mut Text, &Transform, &mut Visibility), With<WallMarker>>,
) {
    let tile_size = 32.0;

    for (mut text, transform, mut visibility) in query.iter_mut() {
        let x = (transform.translation.x / tile_size).round() as i32;
        let y = (transform.translation.y / tile_size).round() as i32;
        let idx = (y * map.width + x) as usize;

        if map.visible_tiles[idx] {
            // NA LUZ: Cor de tocha (amarelado/quente)
            *visibility = Visibility::Visible;
            text.sections[0].style.color = Color::rgb(1.0, 0.9, 0.5);
        } else if map.explored_tiles[idx] {
            // NA MEMÓRIA: Já vi antes, mas agora está no escuro (cinza azulado)
            *visibility = Visibility::Visible;
            text.sections[0].style.color = Color::rgb(1.0, 0.0, 0.0);
            // text.sections[0].style.color = Color::rgb(0.15, 0.15, 0.25);
        } else {
            // NUNCA VISTO: Escuridão total
            *visibility = Visibility::Hidden;
        }
    }
}