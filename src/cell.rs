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
      .with_system(Cell::run_generation.label("run_generation"))
      .with_system(Cell::despawn_dead_zones.after("run_generation"))
    )
    .add_system(Cell::highlight)
    .add_system(Cell::spawn);
  }
}

const TIME_STEP: f64 = 1./5.;
const CELL_GRID_SIZE: f32 = 16.;
const CELL_SIZE: Vec3 = const_vec3!([0.25, 0.25, 0.]);

#[derive(Component, Debug)]
struct Cell {
  id: (i32, i32),
  populated: bool,
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

    commands.insert_resource(NeighborMap(HashMap::new()));

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
    mut commands: Commands,
    game_rules: Res<GameRules>,
    cell_texture: Res<CellTexture>,
    mut neighbor_map: ResMut<NeighborMap>,
    mut cells: Query<(&mut Cell, &mut Visibility)>,
  ) {
    if !game_rules.running {
      return;
    }
    let mut neighbor_map = &mut neighbor_map.0;
    let mut old_neighbor_map = HashMap::new();
    old_neighbor_map.clone_from(&neighbor_map);

    for (mut cell, mut visibility) in cells.iter_mut() {
      if let Some(neighbor) = old_neighbor_map.get(&cell.id) {
        if cell.is_alive() {
          if !game_rules.survive.contains(&neighbor.neighbor_count) {
            cell.die();
            Self::update_neighborhood(
              &mut commands,
              &cell_texture.0,
              &mut neighbor_map,
              cell.id,
              true,
            );
          }
        } else {
          if game_rules.revive.contains(&neighbor.neighbor_count) {
            cell.revive();
            Self::update_neighborhood(
              &mut commands,
              &cell_texture.0,
              &mut neighbor_map,
              cell.id,
              false,
            );
          }
        }
      }
      visibility.is_visible = cell.is_alive();
    }
  }

  fn despawn_dead_zones(
    mut commands: Commands,
    game_rules: Res<GameRules>,
    mut neighbor_map: ResMut<NeighborMap>,
    cells: Query<(Entity, &Cell)>,
  ) {
    if !game_rules.running {
      return;
    }
    let neighbor_map = &mut neighbor_map.0;
    cells.for_each(|(cell_entity, cell)| {
      if cell.is_dead() {
        if let Some(neighbor) = neighbor_map.get(&cell.id) {
          if neighbor.neighbor_count == 0 {
            commands.entity(cell_entity).despawn();
            neighbor_map.remove(&cell.id);
          }
        }
      }
    })
  }

  fn keyboard_input(
    keyboard: Res<Input<KeyCode>>,
    mut game_rules: ResMut<GameRules>,
    neighbor_map: Res<NeighborMap>,
    cells: Query<&Cell>,
    cell_frame: Query<&Transform, With<CellFrame>>,
  ) {
    if keyboard.just_pressed(KeyCode::Space) {
      game_rules.running = !game_rules.running;
    } else if keyboard.just_pressed(KeyCode::P) {
      let neighbor_map = &neighbor_map.0;
      println!("-------- DEBUG --------");
      let mut entity_count = 0;
      cells.for_each(|cell| {
        if neighbor_map.contains_key(&cell.id) {
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
    mut neighbor_map: ResMut<NeighborMap>,
    cell_frame: Query<&Transform, With<CellFrame>>,
    mut cells: Query<(&mut Cell, &mut Visibility), With<Cell>>,
  ) {
    if !mouse.pressed(MouseButton::Left) {
      return
    }

    let mut neighbor_map = &mut neighbor_map.0;
    let cell_frame = cell_frame.single();
    let cell_id = (
      cell_frame.translation.x as i32,
      cell_frame.translation.y as i32,
    );

    let cell = match neighbor_map.get(&cell_id) {
      Some(neighbor) => {
        match cells.get_mut(neighbor.entity) {
          Ok(cell_match) => Some(cell_match),
          Err(_) => None,
        }
      },
      None => None,
    };

    match cell {
      Some((mut cell, mut cell_shape)) => {
        if cell.is_dead() {
          cell.revive();
          cell_shape.is_visible = true;
          Self::update_neighborhood(
            &mut commands,
            &cell_texture.0,
            &mut neighbor_map,
            cell_id,
            false,
          );
        }
      }
      None => {
        Self::update_neighborhood(
          &mut commands,
          &cell_texture.0,
          &mut neighbor_map,
          cell_id,
          false,
        );
        let entity = Self::create_cell_entity(
          &mut commands,
          &cell_texture.0,
          Cell { id: cell_id, populated: true },
          cell_id,
        );
        neighbor_map.insert(cell_id, Neighbor { entity, neighbor_count: 0 });
      }
    }
  }

  /// Create a new neighborhood using the cell with cell_id as the center
  /// If the neighborhood already exists, update every neighbors count
  fn update_neighborhood(
    mut commands: &mut Commands,
    cell_texture: &Handle<Image>,
    neighbor_map: &mut HashMap<(i32, i32), Neighbor>,
    cell_id: (i32, i32),
    is_cell_dead: bool,
  ) {
    for id in Cell::gen_neighbor_ids(cell_id) {
      match neighbor_map.get_mut(&id) {
        Some(mut neighbor) => {
          neighbor.neighbor_count = if is_cell_dead {
            neighbor.neighbor_count - 1
          } else {neighbor.neighbor_count + 1};
        }
        None => {
          // Create a dummy empty cell when there's no cell in this space
          let entity = Self::create_cell_entity(
            &mut commands,
            cell_texture,
            Cell { id, populated: false },
            id,
          );
          neighbor_map.insert(id, Neighbor {
            entity,
            neighbor_count: if is_cell_dead {0} else {1},
          });
        }
      }
    }
  }

  fn create_cell_entity(
    commands: &mut Commands,
    cell_texture: &Handle<Image>,
    cell: Cell,
    cell_id: (i32, i32),
  ) -> Entity {
    let mut entity_cmds = commands.spawn();
    let visibility = Visibility { is_visible: cell.populated };
    let entity = entity_cmds.id();
    entity_cmds
    .insert(cell)
    .insert_bundle(SpriteBundle {
      transform: Transform {
        scale: CELL_SIZE,
        translation: Vec3::new(cell_id.0 as f32, cell_id.1 as f32, 1.),
        ..default()
      },
      texture: cell_texture.clone(),
      visibility,
      ..default()
    });
    entity
  }
}

#[derive(Component)]
struct CellFrame;

struct CellTexture(Handle<Image>);

struct GameRules {
  running: bool,
  survive: Range<u8>,
  revive: Range<u8>,
}

struct NeighborMap(HashMap<(i32, i32), Neighbor>);

#[derive(Debug, Clone)]
struct Neighbor {
  entity: Entity,
  neighbor_count: u8,
}

impl Cell {
  pub fn gen_neighbor_ids((x, y): (i32, i32)) -> [(i32, i32); 8] {
    let mut ids = [(0, 0); 8];
    let mut idx = 0;
    for dy in (y - 16 ..= y + 16).step_by(16) {
      for dx in (x - 16 ..= x + 16).step_by(16) {
        if !(x == dx && y == dy) {
          ids[idx] = (dx, dy);
          idx = idx + 1;
        }
      }
    }
    ids
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
