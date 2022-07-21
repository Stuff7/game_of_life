use std::collections::HashMap;
use std::ops::Range;
use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;
use bevy::math::const_vec3;
use bevy::core::FixedTimestep;
use bevy::render::view::visibility;

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

const TIME_STEP: f64 = 1./5.;
const CELL_GRID_SIZE: f32 = 16.;
const CELL_SIZE: Vec3 = const_vec3!([0.25, 0.25, 0.]);

#[derive(Component, Debug)]
struct Cell {
  populated: bool,
  living_neighbors: u8,
  neighbors: Option<[Entity; 8]>,
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
    mut cells: Query<(&mut Cell, &mut Visibility), With<SubjectToRules>>,
  ) {
    if !game_rules.running {
      return;
    }

    let mut counts: Vec<Option<u8>> = Vec::new();
    cells.for_each(|(cell, _)| {
      if cell.neighbors.is_none() {
        counts.push(None);
        return;
      }

      let neighbors = cell.neighbors.unwrap();
      let neighbors = cells.get_many(neighbors);
      if neighbors.is_err() {
        counts.push(None);
        return;
      }

      // Count living neighbors
      let neighbors = neighbors.unwrap();
      let mut count: u8 = 0;
      for (neighbor, _) in neighbors {
        if neighbor.is_alive() {
          count = count + 1;
        }
      }
      counts.push(Some(count));
    });

    for (
      i,
      (mut cell, mut visibility),
    ) in cells.iter_mut().enumerate() {
      let mut living_neighbors: &u8 = &0;
      let count = counts.get(i);
      if let Some(count) = count {
        if let Some(count) = count {
          living_neighbors = count;
        }
      }
      if cell.is_alive() {
        if !game_rules.survive.contains(living_neighbors) {
          cell.die();
        }
      } else {
        if game_rules.revive.contains(living_neighbors) {
          cell.revive();
        }
      }
      visibility.is_visible = cell.is_alive();
    }
  }

  fn keyboard_input(
    keyboard: Res<Input<KeyCode>>,
    mut game_rules: ResMut<GameRules>,
    cells: Query<(Entity, &Cell), With<Cell>>,
  ) {
    if keyboard.just_pressed(KeyCode::Space) {
      game_rules.running = !game_rules.running;
    }else if keyboard.just_pressed(KeyCode::P) {
      println!("-------- CELLS --------");
      for (entity, cell) in cells.iter() {
        println!("{entity:?} -> {cell:?}\n");
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
    mut entity_map: ResMut<EntityMap>,
    cell_frame: Query<&Transform, With<CellFrame>>,
    mut cells: Query<(&mut Cell, &mut Visibility), With<Cell>>,
  ) {
    if !mouse.pressed(MouseButton::Left) {
      return
    }

    let mut entity_map = &mut entity_map.0;
    let cell_frame = cell_frame.single();
    let cell_id = (
      cell_frame.translation.x as i32,
      cell_frame.translation.y as i32,
    );

    let cell = match entity_map.get(&cell_id) {
      Some(entity) => {
        match cells.get_mut(*entity) {
          Ok(cell_match) => Some(cell_match),
          Err(_) => None,
        }
      },
      None => None,
    };

    match cell {
      Some((mut cell, mut cell_shape)) => {
        cell.revive();
        cell_shape.is_visible = true;
        if cell.neighbors.is_none() {
          cell.neighbors = Some(Self::get_neighborhood(
            &mut commands,
            &cell_texture.0,
            &mut entity_map,
            cell_id,
          ));
        }
      }
      None => {
        let neighbors = Some(Self::get_neighborhood(
          &mut commands,
          &cell_texture.0,
          &mut entity_map,
          cell_id,
        ));
        Self::create_cell_entity(
          &mut commands,
          &cell_texture.0,
          Cell {
            populated: true,
            living_neighbors: 0,
            neighbors,
          },
          &mut entity_map,
          cell_id,
        );
      }
    }
  }

  /// Gets neighborhood for cell_id and creates missing neighbors if they don't exist
  fn get_neighborhood(
    mut commands: &mut Commands,
    cell_texture: &Handle<Image>,
    mut entity_map: &mut HashMap<(i32, i32), Entity>,
    cell_id: (i32, i32),
  ) -> [Entity; 8] {
    let neighbor_ids = Cell::gen_neighbor_ids(cell_id);
    let neighbors = neighbor_ids
    .map(|neighbor_id| {
      match entity_map.get(&neighbor_id) {
        Some(neighbor_entity) => *neighbor_entity,
        None => {
          // Create a dummy empty cell when there's no cell in this space
          Self::create_cell_entity(
            &mut commands,
            cell_texture,
            Cell { populated: false, living_neighbors: 0, neighbors: None },
            &mut entity_map,
            neighbor_id,
          )
        }
      }
    });
    neighbors
  }

  fn create_cell_entity(
    commands: &mut Commands,
    cell_texture: &Handle<Image>,
    cell: Cell,
    entity_map: &mut HashMap<(i32, i32), Entity>,
    cell_id: (i32, i32),
  ) -> Entity {
    let mut entity_cmds = commands.spawn();
    let visibility = Visibility { is_visible: cell.populated };
    let entity = entity_cmds.id();
    entity_map.insert(cell_id, entity);
    entity_cmds
    .insert(cell)
    .insert(SubjectToRules)
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

#[derive(Component)]
struct SubjectToRules;

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

  pub fn add_neighbor(&mut self) {
    self.living_neighbors = self.living_neighbors + 1;
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
