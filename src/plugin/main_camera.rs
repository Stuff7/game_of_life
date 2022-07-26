use bevy::prelude::*;
use bevy::input::mouse::{MouseWheel, MouseMotion};
use bevy::math::const_vec2;

const ZOOM_SPEED: f32 = 0.1;
const MIN_ZOOM: f32 = 0.1;
const MAX_ZOOM: f32 = 4.;
const PANNING_SPEED: Vec2 = const_vec2!([8., -8.]);

pub struct MainCameraPlugin;

impl Plugin for MainCameraPlugin {
  fn build(&self, app: &mut App) {
    app
    .add_startup_system(setup)
    .add_system(get_cursor_position)
    .add_system(zoom)
    .add_system(pan);
  }
}

#[derive(Component)]
pub struct MainCamera;

fn setup(mut commands: Commands) {
  commands.spawn()
  .insert_bundle(OrthographicCameraBundle::new_2d())
  .insert(MainCamera);

  commands.insert_resource(Cursor::new());
}

fn zoom(
  mut wheel_evr: EventReader<MouseWheel>,
  cursor: Res<Cursor>,
  mut camera: Query<(
    &mut Transform,
    &mut OrthographicProjection,
  ), With<MainCamera>>,
) {
  let delta_zoom: f32 = wheel_evr.iter().map(|e| e.y).sum();
  if delta_zoom == 0. {
      return;
  }

  let (
    mut position,
    mut camera,
  ) = camera.single_mut();

  camera.scale -= ZOOM_SPEED * delta_zoom * camera.scale;
  camera.scale = camera.scale.clamp(MIN_ZOOM, MAX_ZOOM);

  position.translation = (
    cursor.world_coords - cursor.normp_coords * camera.scale
  ).extend(position.translation.z);
}

fn pan(
  mut motion_evr: EventReader<MouseMotion>,
  mouse: Res<Input<MouseButton>>,
  cursor: Res<Cursor>,
  mut camera: Query<(
    &mut Transform,
    &OrthographicProjection,
  ), With<MainCamera>>,
) {
  if !mouse.pressed(MouseButton::Right) {
    return
  }

  if motion_evr.is_empty() {
    return;
  }

  let (
    mut position,
    camera,
  ) = camera.single_mut();

  for motion in motion_evr.iter() {
    position.translation = ((
      cursor.world_coords - cursor.normp_coords * camera.scale
    ) - motion.delta * PANNING_SPEED).extend(position.translation.z);
  }

}

fn get_cursor_position(
  mut camera: Query<(
    &Transform,
    &OrthographicProjection,
  ), With<MainCamera>>,
  mut cursor: ResMut<Cursor>,
  windows: Res<Windows>,
) {
  let window = windows.get_primary();
  if window.is_none() {
    return;
  }
  let window = window.unwrap();

  let (
    position,
    camera,
  ) = camera.single_mut();

  if let Some(cursor_screen_pos) = window.cursor_position() {
    let window_size = Vec2::new(window.width(), window.height());
    let cam_position = position.translation.truncate();
    // convert screen position to normalized device coordinates
    // [0..resolution] => [-1..1]
    let ndc = (cursor_screen_pos / window_size) * 2. - Vec2::ONE;
    // convert normalized coordinates to normalized pixel coordinates
    // [-1..1] => [-win_size/2..win_size/2]
    cursor.normp_coords = ndc * Vec2::new(camera.right, camera.top);
    // convert to world coordinates
    cursor.world_coords = cursor.normp_coords * camera.scale + cam_position;
  }
}

#[derive(Default)]
pub struct Cursor {
  pub world_coords: Vec2,
  pub normp_coords: Vec2,
}

impl Cursor {
  pub fn new() -> Self {
    Self {
      world_coords: Vec2::new(0., 0.),
      normp_coords: Vec2::new(0., 0.),
    }
  }

  pub fn snap_position(&self, block_size: f32) -> (f32, f32) {
    (
      block_size * (
        self.world_coords.x / block_size
      ).round(),
      block_size * (
        self.world_coords.y / block_size
      ).round(),
    )
  }
}
