use std::collections::HashMap;
use std::ops::Range;
use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;
use bevy::math::const_vec3;
use bevy::core::FixedTimestep;

use crate::main_camera::Cursor;

pub struct CellPlugin;

impl Plugin for CellPlugin {
  fn build(&self, app: &mut App) {
    app
    .add_startup_system(Cell::setup)
    .add_system(Cell::keyboard_input)
    .add_system_set(
      SystemSet::new()
      .with_run_criteria(FixedTimestep::step(TIME_STEP))
      .with_system(Cell::run_generation)
    )
    .add_system(Cell::highlight)
    .add_system(Cell::spawn);
  }
}

const TIME_STEP: f64 = 1./20.;
const CELL_GRID_SIZE: f32 = 16.;
const CELL_SIZE: Vec3 = const_vec3!([0.25, 0.25, 0.]);

#[derive(Component, Debug)]
struct Cell {
  populated: bool,
  living_neighbors: u8,
  neighbors: [Option<Entity>; 8],
}

impl Cell {
  fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let cell_frame_texture = asset_server.load("textures/cell_frame.png");
    let cell_texture = asset_server.load("textures/cell.png");
    commands.insert_resource(CellTexture(cell_texture));

    commands.insert_resource(GameRules {
      running: false,
      survive: (2..4),
      revive: (3..4),
    });

    commands.insert_resource(EntityMap(HashMap::new()));

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

  fn run_generation(
    game_rules: Res<GameRules>,
    mut cells: Query<(&mut Cell, &mut Visibility), With<Cell>>,
  ) {
    if !game_rules.running {
      return;
    }

    for (mut cell, mut visibility) in cells.iter_mut() {
      if cell.is_alive() {
        if !game_rules.survive.contains(&cell.living_neighbors) {
          cell.die();
        }
      } else {
        if game_rules.revive.contains(&cell.living_neighbors) {
          cell.revive();
        }
      }
      visibility.is_visible = cell.is_alive();
    }
  }

  fn keyboard_input(
    keyboard: Res<Input<KeyCode>>,
    mut game_rules: ResMut<GameRules>,
    cells: Query<&Cell>,
  ) {
    if keyboard.just_pressed(KeyCode::Space) {
      game_rules.running = !game_rules.running;
    }else if keyboard.just_pressed(KeyCode::P) {
      println!("-------- CELLS --------");
      for cell in cells.iter() {
        println!("{cell:?}\n");
      }
      println!("-------- -END- --------");
    }
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
    cell_texture: Res<CellTexture>,
    mut entity_id_map: ResMut<EntityMap>,
    cell_frame: Query<&Transform, With<CellFrame>>,
    mut cells: Query<&mut Cell>,
  ) {
    if !mouse.pressed(MouseButton::Left) {
      return
    }

    let entity_id_map = &mut entity_id_map.0;
    let cell_frame = cell_frame.single();
    let cell_id = (
      cell_frame.translation.x as i32,
      cell_frame.translation.y as i32,
    );

    let cell = match entity_id_map.get(&cell_id) {
      Some(entity) => {
        match cells.get_mut(*entity) {
          Ok(cell_match) => Some(cell_match),
          Err(_) => None,
        }
      },
      None => None,
    };

    match cell {
      Some(mut cell) => {
        cell.revive();
      }
      None => {
        let mut entity_cmds = commands.spawn();
        let entity = entity_cmds.id();

        entity_id_map.insert(cell_id, entity);

        let mut living_neighbors: u8 = 0;
        let neighbors = Cell::gen_neighbor_ids(cell_id)
        .map(|neighbor_id| {
          let maybe_entity = entity_id_map.get(&neighbor_id);
          if let Some(neighbor_entity) = maybe_entity {
            match cells.get_mut(*neighbor_entity) {
              Ok(mut neighbor_cell) => {
                neighbor_cell.add_neighbor(entity);
                if neighbor_cell.is_alive() {
                  living_neighbors = living_neighbors + 1;
                }
              }
              Err(_) => {}
            }
            return Some(*neighbor_entity)
          }
          None
        });

        let cell = Cell {
          living_neighbors,
          neighbors,
          populated: true,
        };
        entity_cmds
        .insert(cell)
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
}

struct CellTexture(Handle<Image>);

#[derive(Component)]
struct CellFrame;

struct GameRules {
  running: bool,
  survive: Range<u8>,
  revive: Range<u8>,
}

struct EntityMap(HashMap<(i32, i32), Entity>);

impl Cell {
  pub fn gen_neighbor_ids((x, y): (i32, i32)) -> [(i32, i32); 8] {
    let mut ids = [(0, 0); 8];
    let mut idx = 0;
    for dy in (y - 16..=y + 16).step_by(16) {
      for dx in (x - 16..=x + 16).step_by(16) {
        if !(x == dx && y == dy) {
          ids[idx] = (dx, dy);
          idx = idx + 1;
        }
      }
    }
    ids
  }

  pub fn add_neighbor(&mut self, entity: Entity) {
    self.living_neighbors = self.living_neighbors + 1;
    let maybe_spot = self.neighbors.iter()
    .position(|neighbor| neighbor.is_none());

    if let Some(spot) = maybe_spot {
      self.neighbors[spot] = Some(entity);
    }
  }

  pub fn remove_neighbor(&mut self) {
    self.living_neighbors = self.living_neighbors - 1;
  }

  pub fn is_alive(&self) -> bool {
    self.populated
  }

  pub fn is_dead(&self) -> bool {
    !self.populated
  }

  pub fn die(&mut self) {
    self.populated = false;
  }

  pub fn revive(&mut self) {
    self.populated = true;
  }
}
