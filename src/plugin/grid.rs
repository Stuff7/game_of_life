use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;
use bevy::math::const_vec3;

use super::main_camera::Cursor;

const CELL_IMG_SIZE: f32 = 64.;
const CELL_PX_SIZE: f32 = 16.;
const CELL_TRANSFORM_SIZE: f32 = CELL_PX_SIZE / CELL_IMG_SIZE;
pub const CELL_SIZE: Vec3 = const_vec3!([CELL_TRANSFORM_SIZE, CELL_TRANSFORM_SIZE, 0.]);
const SELECTOR_STARTING_POSITION: Vec3 = const_vec3!([0., 0., 1.]);

pub struct GridPlugin;

impl Plugin for GridPlugin {
  fn build(&self, app: &mut App) {
    app
    .add_startup_system(setup)
    .add_system(move_selector);
  }
}

#[derive(Component)]
pub struct GridSelector;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
  let grid_select_texture = asset_server.load("textures/grid_selector.png");
  
  commands
  .spawn()
  .insert(GridSelector)
  .insert_bundle(SpriteBundle {
    transform: Transform {
      scale: CELL_SIZE,
      translation: SELECTOR_STARTING_POSITION,
      ..default()
    },
    texture: grid_select_texture,
    ..default()
  });
}

fn move_selector(
  motion_evr: EventReader<MouseMotion>,
  cursor: Res<Cursor>,
  mut grid_selector: Query<&mut Transform, With<GridSelector>>,
) {
  if motion_evr.is_empty() {
    return;
  }
  
  let mut grid_selector = grid_selector.single_mut();
  let snap_position = cursor.snap_position(CELL_PX_SIZE);
  
  grid_selector.translation.x = snap_position.0;
  grid_selector.translation.y = snap_position.1;
}
