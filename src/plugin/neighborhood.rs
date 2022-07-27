use std::collections::HashMap;
use bevy::prelude::*;
use bevy::core::FixedTimestep;

use super::cell::Cell;
use crate::Game;

const TIME_STEP: f64 = 1./5.;

pub struct NeighborhoodPlugin;

impl Plugin for NeighborhoodPlugin {
  fn build(&self, app: &mut App) {
    app
    .add_startup_system(setup)
    .add_system_set(
      SystemSet::new()
      .label("neighbor-counting")
      .with_system(count.after("neighborhood-spawn"))
      .with_system(clean_dead)
    )
    .add_system(
      run_generation
      .with_run_criteria(FixedTimestep::step(TIME_STEP))
      .after("neighbor-counting")
    );
  }
}

pub type NeighborID = (i32, i32);

#[derive(Default)]
pub struct Neighborhood {
  pub map: HashMap<NeighborID, Neighbor>,
}

impl Neighborhood {
  pub fn add_neighbor(&mut self, id: NeighborID, entity: Entity) {
    let mut new_neighbor = Neighbor::new(id, entity);
    for (i, neighbor_id) in new_neighbor.neighbors_ids.iter().enumerate() {
      new_neighbor.neighbors[i] = match self.map.get_mut(&neighbor_id) {
        Some(neighbor) => {
          if let Some(spot) = neighbor.neighbors.iter()
          .position(|n| n.is_none()) {
            neighbor.neighbors[spot] = Some(new_neighbor.entity);
          }
          Some(neighbor.entity)
        }
        None => None
      }
    }
    self.map.insert(id, new_neighbor);
  }

  pub fn remove_neighbor(&mut self, id: NeighborID) -> Option<Neighbor> {
    match self.map.remove(&id) {
      Some(removed_neighbor) => {
        for neighbor_id in removed_neighbor.neighbors_ids {
          if let Some(neighbor) = self.map.get_mut(&neighbor_id) {
            if let Some(spot) = neighbor.neighbors.iter().position(|maybe_entity| {
              if maybe_entity.is_none() {
                return false
              }
              let entity = maybe_entity.unwrap();
              entity == removed_neighbor.entity
            }) {
              neighbor.neighbors[spot] = None;
            }
          }
        }
        Some(removed_neighbor)
      }
      None => None
    }
  }
}

pub struct Neighbor {
  pub neighbors: [Option<Entity>; 8],
  pub neighbors_ids: [NeighborID; 8],
  pub neighbor_count: u8,
  pub entity: Entity,
}

impl Neighbor {
  pub fn new(id: NeighborID, entity: Entity) -> Self {
    Self {
      neighbors_ids: Neighbor::get_neighbor_ids(id),
      neighbors: [None; 8],
      neighbor_count: 0,
      entity,
    }
  }

  fn get_neighbor_ids((x, y): NeighborID) -> [NeighborID; 8] {
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
}

#[derive(Component)]
struct DeadNeighborhood;

fn setup(mut commands: Commands) {
  commands.insert_resource(Neighborhood::default());
}

fn count(
  mut commands: Commands,
  mut neighborhood: ResMut<Neighborhood>,
  cells: Query<&Cell>,
) {
  for neighbor in neighborhood.map.values_mut() {
    neighbor.neighbor_count = 0;
    for inner_neighbor in neighbor.neighbors {
      if inner_neighbor.is_none() {
        continue
      }
      if let Ok(cell) = cells.get(inner_neighbor.unwrap()) {
        if cell.is_alive {
          neighbor.neighbor_count = neighbor.neighbor_count + 1;
        }
      }
    }
    if let Ok(cell) = cells.get(neighbor.entity) {
      if cell.is_dead() && neighbor.neighbor_count == 0 {
        commands.entity(neighbor.entity).insert(DeadNeighborhood);
      }
    }
  }
}

fn clean_dead(
  mut commands: Commands,
  mut neighborhood: ResMut<Neighborhood>,
  dead_cells: Query<&Cell, With<DeadNeighborhood>>,
) {
  dead_cells.for_each(|cell| {
    if let Some(neighbor) = neighborhood.remove_neighbor(cell.id) {
      commands.entity(neighbor.entity).despawn();
    }
  })
}

fn run_generation(
  game: Res<Game>,
  neighborhood: ResMut<Neighborhood>,
  mut cells: Query<(&mut Cell, &mut Visibility)>,
) {
  if !game.running {
    return;
  }

  for (mut cell, mut visibility) in cells.iter_mut() {
    if let Some(neighbor) = neighborhood.map.get(&cell.id) {
      if cell.is_alive {
        if !game.rules.survive.contains(&neighbor.neighbor_count) {
          cell.die();
        }
      } else {
        if game.rules.revive.contains(&neighbor.neighbor_count) {
          cell.revive();
        }
      }
    }
    visibility.is_visible = cell.is_alive;
  }
}
