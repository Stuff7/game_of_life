use bevy::prelude::*;
use super::grid::GridSelector;
use super::grid::CELL_SIZE;
use super::neighborhood::{NeighborID, Neighborhood};

pub struct CellPlugin;

impl Plugin for CellPlugin {
  fn build(&self, app: &mut App) {
    app
    .add_startup_system(setup)
    .add_system(spawn);
  }
}

#[derive(Component)]
pub struct CellTexture {
  pub handle: Handle<Image>,
}

#[derive(Component)]
pub struct Cell {
  pub id: NeighborID,
  pub is_alive: bool,
}

impl Cell {
  pub fn revive(&mut self) {
    self.is_alive = true;
  }

  pub fn die(&mut self) {
    self.is_alive = false;
  }

  pub fn is_dead(&self) -> bool {
    !self.is_alive
  }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
  let cell_texture = asset_server.load("textures/cell.png");
  commands.insert_resource(CellTexture { handle: cell_texture });
}

fn spawn(
  mut commands: Commands,
  mouse: Res<Input<MouseButton>>,
  cell_texture: Res<CellTexture>,
  mut neighborhood: ResMut<Neighborhood>,
  cell_frame: Query<&Transform, With<GridSelector>>,
  mut cells: Query<(&mut Cell, &mut Visibility), With<Cell>>,
) {
  if !mouse.pressed(MouseButton::Left) {
    return
  }

  let cell_frame = cell_frame.single();
  let cell_id = (
    cell_frame.translation.x as i32,
    cell_frame.translation.y as i32,
  );

  let cell = match neighborhood.map.get(&cell_id) {
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
      }
    }
    None => {
      let entity = create_cell_entity(
        &mut commands,
        &cell_texture.handle,
        Cell { id: cell_id, is_alive: true },
      );
      neighborhood.add_neighbor(cell_id, entity);
    }
  }
}

pub fn create_cell_entity(
  commands: &mut Commands,
  cell_texture: &Handle<Image>,
  cell: Cell,
) -> Entity {
  let mut entity_cmds = commands.spawn();
  let visibility = Visibility { is_visible: cell.is_alive };
  let entity = entity_cmds.id();
  entity_cmds
  .insert_bundle(SpriteBundle {
    transform: Transform {
      scale: CELL_SIZE,
      translation: Vec3::new(cell.id.0 as f32, cell.id.1 as f32, 1.),
      ..default()
    },
    texture: cell_texture.clone(),
    visibility,
    ..default()
  })
  .insert(cell);
  entity
}
