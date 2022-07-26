mod plugin;

use std::ops::RangeInclusive;
use bevy::prelude::*;

const BACKGROUND_COLOR: Color = Color::rgb(0.1, 0.1, 0.1);

pub struct Game {
  running: bool,
  rules: Rules,
}

pub struct Rules {
  survive: RangeInclusive<u8>,
  revive: RangeInclusive<u8>,
}

fn main() {
  App::new()
  .insert_resource(WindowDescriptor {
    title: "Conway's Game Of Life".to_string(),
    ..default()
  })
  .add_plugins(DefaultPlugins)
  .insert_resource(ClearColor(BACKGROUND_COLOR))
  .add_startup_system(setup)
  .add_system(debug)
  .add_plugin(plugin::MainCamera)
  .add_plugin(plugin::Grid)
  .add_plugin(plugin::Cell)
  .add_plugin(plugin::Neighborhood)
  .run();
}

fn setup(mut commands: Commands) {
  commands.insert_resource(Game {
    running: false,
    rules: Rules {
      survive: (2..=3),
      revive: (3..=3),
    },
  });
}

fn debug(
  keyboard: Res<Input<KeyCode>>,
  mut game: ResMut<Game>,
  neighborhood: Res<plugin::neighborhood::Neighborhood>,
  cells: Query<&plugin::cell::Cell>,
  cell_frame: Query<&Transform, With<plugin::grid::GridSelector>>,
) {
  if keyboard.just_pressed(KeyCode::Space) {
    game.running = !game.running;
  } else if keyboard.just_pressed(KeyCode::P) {
    println!("-------- DEBUG --------");
    let mut entity_count = 0;
    cells.for_each(|cell| {
      if neighborhood.map.contains_key(&cell.id) {
        entity_count = entity_count + 1;
      }
    });
    println!("Entities: {entity_count}");
    println!("-------- -END- --------");
  } else if keyboard.just_pressed(KeyCode::C) {
    if let Ok(cell_frame) = cell_frame.get_single() {
      println!("COORDS: ({}, {})", cell_frame.translation.x, cell_frame.translation.y);
    }
  }
}
