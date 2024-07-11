use crate::{
	components::{Bar, BarValues},
	traits::{GetScreenPosition, UIBarUpdate},
};
use bevy::{
	ecs::{
		component::Component,
		entity::Entity,
		query::Without,
		system::{Commands, Query},
	},
	transform::components::GlobalTransform,
};

type WithOutBarValues<'a, TSource> = (Entity, &'a GlobalTransform, &'a TSource, &'a mut Bar);
type WithBarValues<'a, TSource> = (
	&'a GlobalTransform,
	&'a TSource,
	&'a mut Bar,
	&'a mut BarValues<TSource>,
);

pub(crate) fn bar<TSource: Component, TCamera: Component + GetScreenPosition>(
	commands: Commands,
	without_bar_values: Query<WithOutBarValues<TSource>, Without<BarValues<TSource>>>,
	with_bar_values: Query<WithBarValues<TSource>>,
	camera: Query<(&TCamera, &GlobalTransform)>,
) where
	BarValues<TSource>: UIBarUpdate<TSource>,
{
	let Ok((camera, camera_transform)) = camera.get_single() else {
		return;
	};
	add_bar_values(commands, without_bar_values, camera, camera_transform);
	update_bar_values(with_bar_values, camera, camera_transform);
}

fn add_bar_values<TSource: Component, TCamera: Component + GetScreenPosition>(
	mut commands: Commands,
	mut agents: Query<WithOutBarValues<TSource>, Without<BarValues<TSource>>>,
	camera: &TCamera,
	camera_transform: &GlobalTransform,
) where
	BarValues<TSource>: UIBarUpdate<TSource>,
{
	for (id, transform, display, mut bar) in &mut agents {
		let world_position = transform.translation() + bar.offset;
		bar.position = camera.get_screen_position(camera_transform, world_position);
		let mut bar_values = BarValues::<TSource>::default();
		bar_values.update(display);
		commands.entity(id).insert(bar_values);
	}
}

fn update_bar_values<TSource: Component, TCamera: Component + GetScreenPosition>(
	mut agents: Query<WithBarValues<TSource>>,
	camera: &TCamera,
	camera_transform: &GlobalTransform,
) where
	BarValues<TSource>: UIBarUpdate<TSource>,
{
	for (transform, display, mut bar, mut bar_values) in &mut agents {
		let world_position = transform.translation() + bar.offset;
		bar.position = camera.get_screen_position(camera_transform, world_position);
		bar_values.update(display);
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

	#[derive(Component, Default)]
	pub struct _Source {
		current: u8,
		max: u8,
	}

	impl UIBarUpdate<_Source> for BarValues<_Source> {
		fn update(&mut self, value: &_Source) {
			self.current = value.current as f32;
			self.max = value.max as f32;
		}
	}

	fn setup(camera: Option<(_Camera, GlobalTransform)>) -> App {
		let mut app = App::new();
		app.add_systems(Update, bar::<_Source, _Camera>);

		match camera {
			None => {
				let mut camera = _Camera::default();
				camera
					.mock
					.expect_get_screen_position()
					.return_const(Vec2::default());
				app.world_mut().spawn((camera, GlobalTransform::default()));
			}
			Some(camera) => {
				app.world_mut().spawn(camera);
			}
		}

		app
	}

	#[test]
	fn add_new_bar_values_when_new() {
		let mut app = setup(None);
		let agent = app
			.world_mut()
			.spawn((
				GlobalTransform::default(),
				Bar::new(Vec3::default(), 0.),
				_Source::default(),
			))
			.id();

		app.update();

		let agent = app.world().entity(agent);

		assert!(agent.contains::<BarValues<_Source>>());
	}

	#[test]
	fn set_position_with_camera_transform_and_agent_position_plus_ui_bar_offset() {
		let camera_transform = GlobalTransform::from_xyz(4., 5., 6.);
		let offset = Vec3::new(1., 2., 3.);
		let mut camera = _Camera::default();
		camera
			.mock
			.expect_get_screen_position()
			.times(1)
			.with(eq(camera_transform), eq(Vec3::new(5., 3., 9.) + offset))
			.return_const(Vec2::default());

		let mut app = setup(Some((camera, camera_transform)));

		app.world_mut().spawn((
			GlobalTransform::from_xyz(5., 3., 9.),
			Bar::new(offset, 0.),
			_Source::default(),
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
			.world_mut()
			.spawn((
				GlobalTransform::default(),
				Bar::default(),
				_Source::default(),
			))
			.id();

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(
			Some(Some(Vec2::new(42., 24.))),
			agent.get::<Bar>().map(|b| b.position)
		);
	}

	#[test]
	fn set_bar_values_current_and_max() {
		let mut app = setup(None);

		let agent = app
			.world_mut()
			.spawn((
				GlobalTransform::default(),
				Bar::default(),
				_Source { current: 1, max: 2 },
			))
			.id();

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(
			Some((1., 2.)),
			agent
				.get::<BarValues<_Source>>()
				.map(|b| (b.current, b.max))
		);
	}

	#[test]
	fn set_position_with_camera_transform_and_agent_position_plus_ui_bar_offset_on_update() {
		let camera_transform = GlobalTransform::from_xyz(4., 5., 6.);
		let offset = Vec3::new(11., 12., 13.);
		let mut camera = _Camera::default();
		camera
			.mock
			.expect_get_screen_position()
			.times(2)
			.with(eq(camera_transform), eq(Vec3::new(5., 3., 9.) + offset))
			.return_const(Vec2::default());

		let mut app = setup(Some((camera, camera_transform)));

		app.world_mut().spawn((
			GlobalTransform::from_xyz(5., 3., 9.),
			Bar::new(offset, 0.),
			_Source::default(),
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
			.world_mut()
			.spawn((
				GlobalTransform::default(),
				Bar::default(),
				_Source::default(),
			))
			.id();

		app.update();
		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(
			Some(Some(Vec2::new(22., 33.))),
			agent.get::<Bar>().map(|b| b.position)
		);
	}

	#[test]
	fn update_bar_values_current_and_max() {
		let mut app = setup(None);

		let agent = app
			.world_mut()
			.spawn((
				GlobalTransform::default(),
				Bar::default(),
				_Source { current: 1, max: 2 },
			))
			.id();

		app.update();

		let mut agent_mut = app.world_mut().entity_mut(agent);
		let mut display = agent_mut.get_mut::<_Source>().unwrap();
		display.current = 10;
		display.max = 33;

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(
			Some((10., 33.)),
			agent
				.get::<BarValues<_Source>>()
				.map(|b| (b.current, b.max))
		);
	}
}
