use crate::{
	components::Bar,
	traits::{GetScreenPosition, UIBarOffset, UIBarScale, UIBarUpdate},
};
use bevy::{
	ecs::{
		component::Component,
		entity::Entity,
		query::{Added, With},
		system::{Commands, Query},
	},
	math::Vec3,
	transform::components::GlobalTransform,
};

type NoBarComponents<'a, TDisplay> = (Entity, &'a GlobalTransform, &'a TDisplay);
type BarComponents<'a, TDisplay> = (&'a GlobalTransform, &'a TDisplay, &'a mut Bar<TDisplay>);

pub fn bar<
	TAgent: Component + UIBarOffset<TDisplay> + UIBarScale<TDisplay>,
	TDisplay: Component,
	TCamera: Component + GetScreenPosition,
>(
	commands: Commands,
	without_bar: Query<NoBarComponents<TDisplay>, (With<TAgent>, Added<TDisplay>)>,
	with_bar: Query<BarComponents<TDisplay>, With<TAgent>>,
	camera: Query<(&TCamera, &GlobalTransform)>,
) where
	Bar<TDisplay>: UIBarUpdate<TDisplay>,
{
	let Ok((camera, camera_transform)) = camera.get_single() else {
		return;
	};
	let offset = TAgent::ui_bar_offset();
	add_bars(commands, without_bar, camera, camera_transform, offset);
	update_bars(with_bar, camera, camera_transform, offset);
}

fn add_bars<
	TAgent: Component + UIBarOffset<TDisplay> + UIBarScale<TDisplay>,
	TDisplay: Component,
	TCamera: Component + GetScreenPosition,
>(
	mut commands: Commands,
	agents: Query<NoBarComponents<TDisplay>, (With<TAgent>, Added<TDisplay>)>,
	camera: &TCamera,
	camera_transform: &GlobalTransform,
	offset: Vec3,
) where
	Bar<TDisplay>: UIBarUpdate<TDisplay>,
{
	for (id, transform, display) in &agents {
		let world_position = transform.translation() + offset;
		let screen_position = camera.get_screen_position(camera_transform, world_position);
		let mut bar = Bar::<TDisplay>::new(screen_position, 0., 0., TAgent::ui_bar_scale());
		bar.update(display);
		commands.entity(id).insert(bar);
	}
}

fn update_bars<
	TAgent: Component + UIBarOffset<TDisplay> + UIBarScale<TDisplay>,
	TDisplay: Component,
	TCamera: Component + GetScreenPosition,
>(
	mut agents: Query<BarComponents<TDisplay>, With<TAgent>>,
	camera: &TCamera,
	camera_transform: &GlobalTransform,
	offset: Vec3,
) where
	Bar<TDisplay>: UIBarUpdate<TDisplay>,
{
	for (transform, display, mut bar) in &mut agents {
		let world_position = transform.translation() + offset;
		let screen_position = camera.get_screen_position(camera_transform, world_position);
		bar.position = screen_position;
		bar.update(display);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		math::{Vec2, Vec3},
	};
	use mockall::{automock, predicate::eq};
	use std::collections::VecDeque;

	#[derive(Component, Default)]
	pub struct _Camera {
		pub mock: Mock_Camera,
	}

	#[automock]
	impl GetScreenPosition for _Camera {
		fn get_screen_position(
			&self,
			camera_transform: &GlobalTransform,
			world_position: Vec3,
		) -> Option<Vec2> {
			self.mock
				.get_screen_position(camera_transform, world_position)
		}
	}

	#[derive(Component)]
	pub struct _Agent;

	#[derive(Component, Default)]
	pub struct _Display {
		current: u8,
		max: u8,
	}

	impl UIBarOffset<_Display> for _Agent {
		fn ui_bar_offset() -> Vec3 {
			Vec3::new(1., 2., 3.)
		}
	}

	impl UIBarScale<_Display> for _Agent {
		fn ui_bar_scale() -> f32 {
			1.3
		}
	}

	impl UIBarUpdate<_Display> for Bar<_Display> {
		fn update(&mut self, value: &_Display) {
			self.current = value.current as f32;
			self.max = value.max as f32;
		}
	}

	fn setup(camera: Option<(_Camera, GlobalTransform)>) -> App {
		let mut app = App::new();
		app.add_systems(Update, bar::<_Agent, _Display, _Camera>);

		match camera {
			None => {
				let mut camera = _Camera::default();
				camera
					.mock
					.expect_get_screen_position()
					.return_const(Vec2::default());
				app.world.spawn((camera, GlobalTransform::default()));
			}
			Some(camera) => {
				app.world.spawn(camera);
			}
		}

		app
	}

	#[test]
	fn add_new_bar_when_new() {
		let mut app = setup(None);
		let agent = app
			.world
			.spawn((GlobalTransform::default(), _Agent, _Display::default()))
			.id();

		app.update();

		let agent = app.world.entity(agent);

		assert!(agent.contains::<Bar<_Display>>());
	}

	#[test]
	fn do_not_add_when_display_not_new() {
		let mut app = setup(None);

		let agent = app
			.world
			.spawn((GlobalTransform::default(), _Display::default()))
			.id();

		app.update();

		app.world.entity_mut(agent).insert(_Agent);

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Bar<_Display>>());
	}

	#[test]
	fn create_offset_with_camera_transform_and_agent_position_plus_ui_bar_offset() {
		let camera_transform = GlobalTransform::from_xyz(4., 5., 6.);
		let mut camera = _Camera::default();
		camera
			.mock
			.expect_get_screen_position()
			.times(1)
			.with(
				eq(camera_transform),
				eq(Vec3::new(5., 3., 9.) + _Agent::ui_bar_offset()),
			)
			.return_const(Vec2::default());

		let mut app = setup(Some((camera, camera_transform)));

		app.world.spawn((
			GlobalTransform::from_xyz(5., 3., 9.),
			_Agent,
			_Display::default(),
		));

		app.update();
	}

	#[test]
	fn set_bar_position() {
		let mut camera = _Camera::default();
		camera
			.mock
			.expect_get_screen_position()
			.return_const(Vec2::new(42., 24.));

		let mut app = setup(Some((camera, GlobalTransform::default())));

		let agent = app
			.world
			.spawn((GlobalTransform::default(), _Agent, _Display::default()))
			.id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(Some(Vec2::new(42., 24.))),
			agent.get::<Bar<_Display>>().map(|b| b.position)
		);
	}

	#[test]
	fn set_bar_scale() {
		let mut app = setup(None);

		let agent = app
			.world
			.spawn((GlobalTransform::default(), _Agent, _Display::default()))
			.id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(Some(1.3), agent.get::<Bar<_Display>>().map(|b| b.scale));
	}

	#[test]
	fn set_bar_current_and_max() {
		let mut app = setup(None);

		let agent = app
			.world
			.spawn((
				GlobalTransform::default(),
				_Agent,
				_Display { current: 1, max: 2 },
			))
			.id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some((1., 2.)),
			agent.get::<Bar<_Display>>().map(|b| (b.current, b.max))
		);
	}

	#[test]
	fn create_offset_with_camera_transform_and_agent_position_plus_ui_bar_offset_on_update() {
		let camera_transform = GlobalTransform::from_xyz(4., 5., 6.);
		let mut camera = _Camera::default();
		camera
			.mock
			.expect_get_screen_position()
			.times(2)
			.with(
				eq(camera_transform),
				eq(Vec3::new(5., 3., 9.) + _Agent::ui_bar_offset()),
			)
			.return_const(Vec2::default());

		let mut app = setup(Some((camera, camera_transform)));

		app.world.spawn((
			GlobalTransform::from_xyz(5., 3., 9.),
			_Agent,
			_Display::default(),
		));

		app.update();
		app.update();
	}

	#[test]
	fn update_bar_position() {
		let mut screen_positions = VecDeque::from([Vec2::new(11., 22.), Vec2::new(22., 33.)]);
		let mut camera = _Camera::default();
		camera
			.mock
			.expect_get_screen_position()
			.returning(move |_, _| screen_positions.pop_front());

		let mut app = setup(Some((camera, GlobalTransform::default())));

		let agent = app
			.world
			.spawn((GlobalTransform::default(), _Agent, _Display::default()))
			.id();

		app.update();
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(Some(Vec2::new(22., 33.))),
			agent.get::<Bar<_Display>>().map(|b| b.position)
		);
	}

	#[test]
	fn update_bar_current_and_max() {
		let mut app = setup(None);

		let agent = app
			.world
			.spawn((
				GlobalTransform::default(),
				_Agent,
				_Display { current: 1, max: 2 },
			))
			.id();

		app.update();

		let mut agent_mut = app.world.entity_mut(agent);
		let mut display = agent_mut.get_mut::<_Display>().unwrap();
		display.current = 10;
		display.max = 33;

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some((10., 33.)),
			agent.get::<Bar<_Display>>().map(|b| (b.current, b.max))
		);
	}
}
