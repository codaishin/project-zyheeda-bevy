use crate::{
	components::{bar::Bar, bar_values::BarValues},
	traits::{GetScreenPosition, UIBarUpdate},
};
use bevy::prelude::*;
use common::{
	traits::{
		accessors::get::{GetProperty, Property, TryApplyOn},
		thread_safe::ThreadSafe,
	},
	zyheeda_commands::ZyheedaCommands,
};

type NewBars<'a, TSource> = (Entity, &'a GlobalTransform, &'a TSource, &'a mut Bar);
type OldBars<'a, TSource, TValue> = (
	&'a GlobalTransform,
	&'a TSource,
	&'a mut Bar,
	&'a mut BarValues<TValue>,
);

#[allow(clippy::type_complexity)]
pub(crate) fn bar<TSource, TValue, TCamera, TMainCameraLabel>(
	commands: ZyheedaCommands,
	without_bar_values: Query<NewBars<TSource>, Without<BarValues<TValue>>>,
	with_bar_values: Query<OldBars<TSource, TValue>>,
	camera: Query<(&TCamera, &GlobalTransform), With<TMainCameraLabel>>,
) where
	TValue: ThreadSafe + for<'a> Property<TValue<'a> = TValue>,
	BarValues<TValue>: UIBarUpdate<TValue>,
	TSource: Component + GetProperty<TValue>,
	TCamera: Component + GetScreenPosition,
	TMainCameraLabel: Component,
{
	let Ok((camera, camera_transform)) = camera.single() else {
		return;
	};
	add_bar_values(commands, without_bar_values, camera, camera_transform);
	update_bar_values(with_bar_values, camera, camera_transform);
}

fn add_bar_values<TSource, TValue, TCamera>(
	mut commands: ZyheedaCommands,
	mut agents: Query<(Entity, &GlobalTransform, &TSource, &mut Bar), Without<BarValues<TValue>>>,
	camera: &TCamera,
	camera_transform: &GlobalTransform,
) where
	TValue: ThreadSafe + for<'a> Property<TValue<'a> = TValue>,
	TSource: Component + GetProperty<TValue>,
	TCamera: Component + GetScreenPosition,
	BarValues<TValue>: UIBarUpdate<TValue>,
{
	for (id, transform, display, mut bar) in &mut agents {
		let world_position = transform.translation() + bar.offset;
		bar.position = camera.get_screen_position(camera_transform, world_position);
		let mut bar_values = BarValues::default();
		bar_values.update(&display.get_property());

		commands.try_apply_on(&id, |mut e| {
			e.try_insert(bar_values);
		});
	}
}

fn update_bar_values<TSource, TValue, TCamera>(
	mut agents: Query<(&GlobalTransform, &TSource, &mut Bar, &mut BarValues<TValue>)>,
	camera: &TCamera,
	camera_transform: &GlobalTransform,
) where
	TValue: ThreadSafe + for<'a> Property<TValue<'a> = TValue>,
	TSource: Component + GetProperty<TValue>,
	TCamera: Component + GetScreenPosition,
	BarValues<TValue>: UIBarUpdate<TValue>,
{
	for (transform, display, mut bar, mut bar_values) in &mut agents {
		let world_position = transform.translation() + bar.offset;
		bar.position = camera.get_screen_position(camera_transform, world_position);
		bar_values.update(&display.get_property());
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		math::{Vec2, Vec3},
		prelude::Bundle,
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::{collections::VecDeque, ops::DerefMut};
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Component, NestedMocks)]
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
	struct _Source(_Value);

	impl Property for _Value {
		type TValue<'a> = Self;
	}

	impl GetProperty<_Value> for _Source {
		fn get_property(&self) -> _Value {
			self.0
		}
	}

	#[derive(Default, Clone, Copy)]
	struct _Value {
		current: u8,
		max: u8,
	}

	#[derive(Component, Default)]
	struct _MainCameraLabel;

	impl UIBarUpdate<_Value> for BarValues<_Value> {
		fn update(&mut self, value: &_Value) {
			self.current = value.current as f32;
			self.max = value.max as f32;
		}
	}

	fn setup<TLabel>(camera: Option<(_Camera, GlobalTransform, TLabel)>) -> App
	where
		TLabel: Bundle + Default,
	{
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, bar::<_Source, _Value, _Camera, _MainCameraLabel>);

		match camera {
			None => {
				app.world_mut().spawn((
					_Camera::new().with_mock(|mock| {
						mock.expect_get_screen_position()
							.return_const(Vec2::default());
					}),
					GlobalTransform::default(),
					TLabel::default(),
				));
			}
			Some(camera) => {
				app.world_mut().spawn(camera);
			}
		}

		app
	}

	#[test]
	fn add_new_bar_values_when_new() {
		let mut app = setup::<_MainCameraLabel>(None);
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

		assert!(agent.contains::<BarValues<_Value>>());
	}

	#[test]
	fn set_position_with_camera_transform_and_agent_position_plus_ui_bar_offset() {
		let camera_transform = GlobalTransform::from_xyz(4., 5., 6.);
		let offset = Vec3::new(1., 2., 3.);
		let mut app = setup(Some((
			_Camera::new().with_mock(|mock| {
				mock.expect_get_screen_position()
					.times(1)
					.with(eq(camera_transform), eq(Vec3::new(5., 3., 9.) + offset))
					.return_const(Vec2::default());
			}),
			camera_transform,
			_MainCameraLabel,
		)));

		app.world_mut().spawn((
			GlobalTransform::from_xyz(5., 3., 9.),
			Bar::new(offset, 0.),
			_Source::default(),
		));

		app.update();
	}

	#[test]
	fn set_bar_position() {
		let mut app = setup(Some((
			_Camera::new().with_mock(|mock| {
				mock.expect_get_screen_position()
					.return_const(Vec2::new(42., 24.));
			}),
			GlobalTransform::default(),
			_MainCameraLabel,
		)));

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
		let mut app = setup::<_MainCameraLabel>(None);

		let agent = app
			.world_mut()
			.spawn((
				GlobalTransform::default(),
				Bar::default(),
				_Source(_Value { current: 1, max: 2 }),
			))
			.id();

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(
			Some((1., 2.)),
			agent.get::<BarValues<_Value>>().map(|b| (b.current, b.max))
		);
	}

	#[test]
	fn set_position_with_camera_transform_and_agent_position_plus_ui_bar_offset_on_update() {
		let camera_transform = GlobalTransform::from_xyz(4., 5., 6.);
		let offset = Vec3::new(11., 12., 13.);
		let mut app = setup(Some((
			_Camera::new().with_mock(|mock| {
				mock.expect_get_screen_position()
					.times(2)
					.with(eq(camera_transform), eq(Vec3::new(5., 3., 9.) + offset))
					.return_const(Vec2::default());
			}),
			camera_transform,
			_MainCameraLabel,
		)));

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
		let mut app = setup(Some((
			_Camera::new().with_mock(|mock| {
				let mut screen_positions =
					VecDeque::from([Vec2::new(11., 22.), Vec2::new(22., 33.)]);

				mock.expect_get_screen_position()
					.returning(move |_, _| screen_positions.pop_front());
			}),
			GlobalTransform::default(),
			_MainCameraLabel,
		)));

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
	fn ignore_cameras_with_wrong_label() {
		#[derive(Component, Default)]
		struct _WrongLabel;

		let camera_transform = GlobalTransform::from_xyz(4., 5., 6.);
		let offset = Vec3::new(1., 2., 3.);
		let mut app = setup(Some((
			_Camera::new().with_mock(|mock| {
				mock.expect_get_screen_position()
					.never()
					.return_const(Vec2::default());
			}),
			camera_transform,
			_WrongLabel,
		)));

		app.world_mut().spawn((
			GlobalTransform::from_xyz(5., 3., 9.),
			Bar::new(offset, 0.),
			_Source::default(),
		));

		app.update();
	}

	#[test]
	fn update_bar_values_current_and_max() {
		let mut app = setup::<_MainCameraLabel>(None);

		let agent = app
			.world_mut()
			.spawn((
				GlobalTransform::default(),
				Bar::default(),
				_Source(_Value { current: 1, max: 2 }),
			))
			.id();

		app.update();

		let mut agent_mut = app.world_mut().entity_mut(agent);
		let mut source = agent_mut.get_mut::<_Source>().unwrap();
		let _Source(values) = source.deref_mut();
		values.current = 10;
		values.max = 33;

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(
			Some((10., 33.)),
			agent.get::<BarValues<_Value>>().map(|b| (b.current, b.max))
		);
	}
}
