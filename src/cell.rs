use std::collections::HashMap;
use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;
use bevy::math::const_vec3;

use crate::main_camera::Cursor;

pub struct CellPlugin;

impl Plugin for CellPlugin {
  fn build(&self, app: &mut App) {
    app
    .add_startup_system(Cell::setup)
    .add_system(Cell::highlight)
    .add_system(Cell::spawn);
  }
}

const CELL_GRID_SIZE: f32 = 16.;
const CELL_SIZE: Vec3 = const_vec3!([0.25, 0.25, 0.]);

#[derive(Component)]
struct Cell;

impl Cell {
  fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let cell_frame_texture = asset_server.load("textures/cell_frame.png");
    let cell_texture = asset_server.load("textures/cell.png");
    commands.insert_resource(CellTexture(cell_texture));

    commands.insert_resource(CellMap(HashMap::new()));

    commands
    .spawn()
    .insert(CellFrame)
    .insert_bundle(SpriteBundle {
      transform: Transform {
        scale: CELL_SIZE,
        translation: Vec3::new(0., 0., 1.),
        ..default()
      },
      texture: cell_frame_texture,
      ..default()
    });
  }

  fn highlight(
    motion_evr: EventReader<MouseMotion>,
    cursor: Res<Cursor>,
    mut cell_frame: Query<&mut Transform, With<CellFrame>>,
  ) {
    if motion_evr.is_empty() {
      return;
    }
  
    let mut cell_frame = cell_frame.single_mut();
  
    cell_frame.translation.x = CELL_GRID_SIZE * (
      cursor.world_coords.x / CELL_GRID_SIZE as f32
    ).round();
    cell_frame.translation.y = CELL_GRID_SIZE * (
      cursor.world_coords.y / CELL_GRID_SIZE as f32
    ).round();
  }

  fn spawn(
    mut commands: Commands,
    mouse: Res<Input<MouseButton>>,
    mut cell_map: ResMut<CellMap>,
    cell_texture: Res<CellTexture>,
    cell_frame: Query<&Transform, With<CellFrame>>,
  ) {
    if !mouse.pressed(MouseButton::Left) {
      return
    }
  
    let cell_frame = cell_frame.single();
    let cell_map = &mut cell_map.0;
    let cell_key = format!(
      "{}:{}",
      cell_frame.translation.x,
      cell_frame.translation.y,
    );

    let cell_is_empty = !*cell_map.get(&cell_key).unwrap_or(&false);
    if cell_is_empty {
      cell_map.insert(cell_key, true);
      commands
      .spawn()
      .insert(Cell)
      .insert_bundle(SpriteBundle {
        transform: Transform {
          scale: CELL_SIZE,
          translation: cell_frame.translation,
          ..default()
        },
        texture: cell_texture.0.clone(),
        ..default()
      });
    }
  }
}

struct CellTexture(Handle<Image>);

struct CellMap(HashMap<String, bool>);

#[derive(Component)]
struct CellFrame;
