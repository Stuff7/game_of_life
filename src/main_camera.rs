use bevy::{prelude::*, input::mouse::{MouseWheel, MouseMotion}, math::const_vec2};

pub struct MainCameraPlugin;

impl Plugin for MainCameraPlugin {
  fn build(&self, app: &mut App) {
    app
    .add_startup_system(MainCamera::setup)
    .add_system(MainCamera::update_world_position_from_cursor)
    .add_system(MainCamera::zoom)
    .add_system(MainCamera::translate);
  }
}

const MIN_ZOOM: f32 = 0.1;
const MAX_ZOOM: f32 = 4.;
const PANNING_SPEED: Vec2 = const_vec2!([8., -8.]);

#[derive(Component)]
pub struct CameraPosition {
  pub world: Vec2,
  pub coords: Vec2,
}

impl CameraPosition {
  pub fn new() -> Self {
    Self { world: Vec2::new(0., 0.), coords: Vec2::new(0., 0.) }
  }
}

#[derive(Component)]
pub struct MainCamera;

impl MainCamera {
  fn setup(mut commands: Commands) {
    commands.spawn()
    .insert_bundle(OrthographicCameraBundle::new_2d())
    .insert(MainCamera)
    .insert(CameraPosition::new());
  }

  fn zoom(
    mut wheel_evr: EventReader<MouseWheel>,
    mut camera: Query<(
      &mut Transform,
      &mut OrthographicProjection,
      &CameraPosition,
    ), With<MainCamera>>,
  ) {
    let delta_zoom: f32 = wheel_evr.iter().map(|e| e.y).sum();
    if delta_zoom == 0. {
        return;
    }

    let (
      mut position,
      mut camera,
      cam_pos,
    ) = camera.single_mut();

    camera.scale -= 0.1 * delta_zoom * camera.scale;
    camera.scale = camera.scale.clamp(MIN_ZOOM, MAX_ZOOM);

    position.translation = (
      cam_pos.world - cam_pos.coords * camera.scale
    ).extend(position.translation.z);
  }

  fn translate(
    mut motion_evr: EventReader<MouseMotion>,
    mouse: Res<Input<MouseButton>>,
    mut camera: Query<(
      &mut Transform,
      &OrthographicProjection,
      &CameraPosition,
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
      cam_pos,
    ) = camera.single_mut();

    for motion in motion_evr.iter() {
      position.translation = ((
        cam_pos.world - cam_pos.coords * camera.scale
      ) - motion.delta * PANNING_SPEED).extend(position.translation.z);
    }

  }

  fn update_world_position_from_cursor(
    mut camera: Query<(
      &Transform,
      &OrthographicProjection,
      &mut CameraPosition
    ), With<MainCamera>>,
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
      mut cam_pos,
    ) = camera.single_mut();

    if let Some(screen_pos) = window.cursor_position() {
      let window_size = Vec2::new(window.width(), window.height());
      let cam_position = position.translation.truncate();
      // convert screen position to normalized device coordinates [0..resolution] => [-1..1]
      let ndc = (screen_pos / window_size) * 2. - Vec2::ONE;
      // convert normalized coordinates to pixel coordinates
      cam_pos.coords = ndc * Vec2::new(camera.right, camera.top);
      // scale pixel coordinates and add the camera position
      cam_pos.world = cam_pos.coords * camera.scale + cam_position;
    }
  }
}
