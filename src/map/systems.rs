use crate::{
    map::components::WallMarker,
    player::components::{Player, Position},
};
use bevy::prelude::*;

use super::components::{Map, TileType};
use crate::map::MapAssets;


pub fn render_world_textured(
    mut commands: Commands,
    map: Res<Map>,
    asset_server: Res<MapAssets>,
    player_query: Query<&Position, With<Player>>,
    tiles_query: Query<Entity, With<TileMarker>>,
) {
    // 1. Falha silenciosamente se o player não existir
    let Ok(player_pos) = player_query.get_single() else { return };

    // 2. Limpa todos os tiles renderizados anteriormente para evitar sobreposição
    for entity in tiles_query.iter() {
        commands.entity(entity).despawn();
    }

    let tile_size = 16.0;
    // Aumentamos o render_dist para garantir que a visão não "corte" abruptamente
    let render_dist = 25; 

    // 3. Iteramos sobre a vizinhança do player baseada nos índices inteiros (grid)
    for y in (player_pos.y - render_dist)..=(player_pos.y + render_dist) {
        for x in (player_pos.x - render_dist)..=(player_pos.x + render_dist) {
            
            // Verifica se as coordenadas estão dentro dos limites do vetor do mapa
            if x < 0 || x >= map.width || y < 0 || y >= map.height {
                continue;
            }

            let idx = (y * map.width + x) as usize;

            // 4. Só renderizamos o que já foi "visto" ou está sendo visto agora
            if map.explored_tiles[idx] {
                let texture = if map.tiles[idx] == TileType::Wall {
                    asset_server.wall_texture.clone()
                } else {
                    asset_server.floor_texture.clone()
                };

                // 5. Lógica de Cores: 
                // Se visível pelo Raycast: Quase branco (deixa a FovMesh brilhar por cima)
                // Se explorado mas fora do cone: Penumbra cinza escura
                let tint = if map.visible_tiles[idx] {
                    Color::rgb(0.9, 0.9, 0.9) 
                } else {
                    Color::rgb(0.2, 0.2, 0.2)
                };

                // 6. Spawn do Sprite
                commands.spawn((
                    SpriteBundle {
                        texture,
                        sprite: Sprite { 
                            color: tint, 
                            // Forçamos o tamanho para 16x16 independente da imagem original
                            custom_size: Some(Vec2::new(tile_size, tile_size)), 
                            ..default() 
                        },
                        // Importante: A posição Z é 0.0 para ficar atrás do Player (Z=2.0) e Luz (Z=1.0)
                        transform: Transform::from_xyz(
                            x as f32 * tile_size, 
                            y as f32 * tile_size, 
                            0.0
                        ),
                        ..default()
                    },
                    TileMarker,
                ));
            }
        }
    }
}

#[derive(Component)]
pub struct TileMarker;

pub fn setup_minimap(mut commands: Commands) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Px(150.0), // Tamanho fixo do minimapa
                height: Val::Px(150.0),
                position_type: PositionType::Absolute,
                right: Val::Px(20.0), // Distância da borda direita
                top: Val::Px(20.0),   // Distância do topo
                border: UiRect::all(Val::Px(2.0)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            background_color: Color::rgba(0.0, 0.0, 0.0, 0.8).into(), // Fundo semi-transparente
            border_color: Color::WHITE.into(),
            ..default()
        })
        .insert(Name::new("MinimapContainer"));
}

pub fn render_fov(
    map: Res<Map>,
    // 1. Precisamos da posição FLOAT do player para calcular a distância exata
    player_query: Query<&Position, With<Player>>,
    mut tile_query: Query<(&mut Text, &Transform, &mut Visibility), With<TileMarker>>,
) {
    // Segurança: se o player não existir, não renderiza o FOV
    let Ok(player_pos) = player_query.get_single() else { return };
    
    let tile_size = 32.0;
    
    // --- CONFIGURAÇÃO DA SUAVE (PENUMBRA) ---
    let max_range = 15.0; // Alcance total da luz (igual ao update_fov)
    let safe_zone = 6.0;  // Até 6 tiles de distância, a luz é 100% forte
    
    // Posição central do player no mundo (em pixels)
    let player_world_x = player_pos.x_float * tile_size;
    let player_world_y = player_pos.y_float * tile_size;

    for (mut text, transform, mut visibility) in tile_query.iter_mut() {
        // Posição do tile no mundo
        let tile_world_x = transform.translation.x;
        let tile_world_y = transform.translation.y;

        // Converte a posição do Transform para índice do Grid (para checar visibilidade lógica)
        let x_grid = (tile_world_x / tile_size).round() as i32;
        let y_grid = (tile_world_y / tile_size).round() as i32;

        // Bounds check (evita crash)
        if x_grid < 0 || x_grid >= map.width || y_grid < 0 || y_grid >= map.height {
            *visibility = Visibility::Hidden;
            continue;
        }

        let idx = (y_grid * map.width + x_grid) as usize;

        // --- LÓGICA DE RENDERIZAÇÃO COM FADE-OUT ---
        
        if map.visible_tiles[idx] {
            // --- NA LUZ (Visão Direta) ---
            *visibility = Visibility::Visible;

            // 2. Cálculo da distância exata (Euclidiana) entre o player e o tile
            let distance = Vec2::new(player_world_x, player_world_y)
                .distance(Vec2::new(tile_world_x, tile_world_y)) / tile_size;

            // 3. Cálculo do Alpha (Opacidade) baseado na distância
            // Se estiver perto (<= safe_zone), alpha é 1.0 (totalmente opaco).
            // Se estiver longe, o alpha diminui gradualmente até 0.0 no max_range.
            let alpha = if distance <= safe_zone {
                1.0
            } else {
                // Interpolação linear inversa: (dist - safe) / (max - safe)
                // Usamos 1.0 - isso para inverter (quanto mais longe, menor o alpha)
                let fade_factor = (distance - safe_zone) / (max_range - safe_zone);
                (1.0 - fade_factor).clamp(0.0, 1.0) // Garante que fique entre 0 e 1
            };

            // Aplica a cor quente com o Alpha calculado (Penumbra)
            //text.sections[0].style.color = Color::rgba(1.0, 0.9, 0.5, alpha);
            text.sections[0].style.color = Color::rgba(1.0, 1.0, 1.0, alpha);
        
        } else if map.explored_tiles[idx] {
            // --- NA MEMÓRIA (Já explorado) ---
            *visibility = Visibility::Visible;
            // Um cinza azulado escuro e fixo para o Fog of War
            text.sections[0].style.color = Color::rgb(0.15, 0.15, 0.25);
        
        } else {
            // --- DESCONHECIDO ---
            *visibility = Visibility::Hidden;
        }
    }
}
