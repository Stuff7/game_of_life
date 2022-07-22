mod main_camera;
mod cell;

use bevy::prelude::*;
use main_camera::MainCameraPlugin;
use cell::CellPlugin;

const BACKGROUND_COLOR: Color = Color::rgb(0.1, 0.1, 0.1);

fn main() {
  App::new()
  .insert_resource(WindowDescriptor {
    title: "Conway's Game Of Life".to_string(),
    ..default()
  })
  .add_plugins(DefaultPlugins)
  .insert_resource(ClearColor(BACKGROUND_COLOR))
  .add_plugin(MainCameraPlugin)
  .add_plugin(CellPlugin)
  .run();
}
