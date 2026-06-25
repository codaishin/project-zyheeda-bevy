use crate::{
	components::{bar::Bar, bar_values::BarValues},
	traits::UIBarUpdate,
};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::{
	traits::{
		accessors::get::{GetContextMut, TryApplyOn, View, ViewField},
		handles_graphics::{CameraHandle, ScreenPosition},
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
pub(crate) fn bar<TSource, TValue, TCamera>(
	commands: ZyheedaCommands,
	without_bar_values: Query<NewBars<TSource>, Without<BarValues<TValue>>>,
	with_bar_values: Query<OldBars<TSource, TValue>>,
	mut camera: StaticSystemParam<TCamera>,
) where
	TValue: ThreadSafe + for<'a> ViewField<TValue<'a> = TValue>,
	BarValues<TValue>: UIBarUpdate<TValue>,
	TSource: Component + View<TValue>,
	TCamera: for<'c> GetContextMut<CameraHandle, TContext<'c>: ScreenPosition>,
{
	let camera = TCamera::get_context_mut(&mut camera, CameraHandle);

	add_bar_values(commands, without_bar_values, &camera);
	update_bar_values(with_bar_values, &camera);
}

fn add_bar_values<TSource, TValue, TCamera>(
	mut commands: ZyheedaCommands,
	mut agents: Query<(Entity, &GlobalTransform, &TSource, &mut Bar), Without<BarValues<TValue>>>,
	camera: &TCamera,
) where
	TValue: ThreadSafe + for<'a> ViewField<TValue<'a> = TValue>,
	TSource: Component + View<TValue>,
	TCamera: ScreenPosition,
	BarValues<TValue>: UIBarUpdate<TValue>,
{
	for (id, transform, display, mut bar) in &mut agents {
		let world_position = transform.translation() + bar.offset;
		bar.position = camera.screen_position(world_position);
		let mut bar_values = BarValues::default();
		bar_values.update(&display.view());

		commands.try_apply_on(&id, |mut e| {
			e.try_insert(bar_values);
		});
	}
}

fn update_bar_values<TSource, TValue, TCamera>(
	mut agents: Query<(&GlobalTransform, &TSource, &mut Bar, &mut BarValues<TValue>)>,
	camera: &TCamera,
) where
	TValue: ThreadSafe + for<'a> ViewField<TValue<'a> = TValue>,
	TSource: Component + View<TValue>,
	TCamera: ScreenPosition,
	BarValues<TValue>: UIBarUpdate<TValue>,
{
	for (transform, display, mut bar, mut bar_values) in &mut agents {
		let world_position = transform.translation() + bar.offset;
		bar.position = camera.screen_position(world_position);
		bar_values.update(&display.view());
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use bevy::{
		app::{App, Update},
		math::{Vec2, Vec3},
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::{collections::VecDeque, ops::DerefMut};
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Resource, NestedMocks)]
	pub struct _Camera {
		pub mock: Mock_Camera,
	}

	#[automock]
	impl ScreenPosition for _Camera {
		fn screen_position(&self, translation: Vec3) -> Option<Vec2> {
			self.mock.screen_position(translation)
		}
	}

	impl ScreenPosition for &mut _Camera {
		fn screen_position(&self, translation: Vec3) -> Option<Vec2> {
			(self as &_Camera).screen_position(translation)
		}
	}

	#[derive(Component, Default)]
	struct _Source(_Value);

	impl ViewField for _Value {
		type TValue<'a> = Self;
	}

	impl View<_Value> for _Source {
		fn view(&self) -> _Value {
			self.0
		}
	}

	#[derive(Default, Clone, Copy)]
	struct _Value {
		current: u8,
		max: u8,
	}

	impl UIBarUpdate<_Value> for BarValues<_Value> {
		fn update(&mut self, value: &_Value) {
			self.current = value.current as f32;
			self.max = value.max as f32;
		}
	}

	fn setup(camera: Option<_Camera>) -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, bar::<_Source, _Value, ResMut<_Camera>>);

		match camera {
			None => {
				app.insert_resource(_Camera::new().with_mock(|mock| {
					mock.expect_screen_position().return_const(Vec2::default());
				}));
			}
			Some(camera) => {
				app.insert_resource(camera);
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

		assert!(agent.contains::<BarValues<_Value>>());
	}

	#[test]
	fn set_position_with_camera_transform_and_agent_position_plus_ui_bar_offset() {
		let offset = Vec3::new(1., 2., 3.);
		let mut app = setup(Some(_Camera::new().with_mock(|mock| {
			mock.expect_screen_position()
				.times(1)
				.with(eq(Vec3::new(5., 3., 9.) + offset))
				.return_const(Vec2::default());
		})));

		app.world_mut().spawn((
			GlobalTransform::from_xyz(5., 3., 9.),
			Bar::new(offset, 0.),
			_Source::default(),
		));

		app.update();
	}

	#[test]
	fn set_bar_position() {
		let mut app = setup(Some(_Camera::new().with_mock(|mock| {
			mock.expect_screen_position()
				.return_const(Vec2::new(42., 24.));
		})));

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
		let offset = Vec3::new(11., 12., 13.);
		let mut app = setup(Some(_Camera::new().with_mock(|mock| {
			mock.expect_screen_position()
				.times(2)
				.with(eq(Vec3::new(5., 3., 9.) + offset))
				.return_const(Vec2::default());
		})));

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
		let mut app = setup(Some(_Camera::new().with_mock(|mock| {
			let mut screen_positions = VecDeque::from([Vec2::new(11., 22.), Vec2::new(22., 33.)]);

			mock.expect_screen_position()
				.returning(move |_| screen_positions.pop_front());
		})));

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
