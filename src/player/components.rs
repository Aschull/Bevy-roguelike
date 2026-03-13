use bevy::prelude::*;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct FacingDirection(pub f32);

#[derive(Component)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}