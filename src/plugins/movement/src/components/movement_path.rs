use crate::{
	components::ongoing_movement::OngoingMovement,
	traits::movement_update::MovementUpdate,
};
use bevy::prelude::*;
use common::{
	tools::{Done, speed::Speed},
	traits::handles_movement::{CurrentMovement, MovementTarget},
	zyheeda_commands::ZyheedaEntityCommands,
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[savable_component(id = "movement path")]
pub struct MovementPath(pub(crate) Mode);

impl MovementPath {
	fn take_next(&mut self) -> Option<MovementTarget> {
		match &mut self.0 {
			Mode::PathTarget(_) => None,
			Mode::Direction(target) => {
				let wp = Some(MovementTarget::Dir(*target));
				self.0 = Mode::PathTarget(None);
				wp
			}
			Mode::Path(path) => path.pop_front().map(MovementTarget::Point),
		}
	}

	pub(crate) fn stop() -> Self {
		Self(Mode::PathTarget(None))
	}

	pub(crate) fn direction(dir: Dir3) -> Self {
		Self(Mode::Direction(dir))
	}

	pub(crate) fn target(pos: Vec3) -> Self {
		Self(Mode::PathTarget(Some(pos)))
	}

	pub(crate) fn path(path: impl Into<VecDeque<Vec3>>) -> Self {
		Self(Mode::Path(path.into()))
	}
}

impl<T> From<T> for MovementPath
where
	T: Into<MovementTarget>,
{
	fn from(value: T) -> Self {
		match value.into() {
			MovementTarget::Dir(direction) => Self::direction(direction),
			MovementTarget::Point(point) => Self::target(point),
		}
	}
}

impl MovementUpdate for MovementPath {
	type TComponents = &'static mut Self;

	fn update(entity: &mut ZyheedaEntityCommands, mut path: Mut<Self>, _: Speed) -> Done {
		match path.take_next() {
			Some(next) => {
				entity.try_insert(OngoingMovement::target(next));
				Done(false)
			}
			None => {
				entity.try_insert(OngoingMovement::Stopped);
				Done(true)
			}
		}
	}

	fn stop(entity: &mut ZyheedaEntityCommands) {
		entity.try_insert(Self::stop());
	}
}

impl CurrentMovement for MovementPath {
	fn current_movement(&self) -> Option<MovementTarget> {
		match &self.0 {
			Mode::Direction(dir) => Some(MovementTarget::Dir(*dir)),
			Mode::PathTarget(target) => target.as_ref().copied().map(MovementTarget::Point),
			Mode::Path(path) => path.back().copied().map(MovementTarget::Point),
		}
	}
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub(crate) enum Mode {
	Direction(Dir3),
	PathTarget(Option<Vec3>),
	Path(VecDeque<Vec3>),
}

#[cfg(test)]
mod test_with_path {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{
		tools::UnitsPerSecond,
		traits::accessors::get::TryApplyOn,
		zyheeda_commands::{ZyheedaCommands, ZyheedaEntityCommands},
	};
	use std::sync::LazyLock;
	use testing::SingleThreadedApp;

	#[derive(Debug, PartialEq, Default)]
	struct _MoveMethod;

	fn system<R>(
		func: impl Fn(&mut ZyheedaEntityCommands, Mut<MovementPath>) -> R,
	) -> impl Fn(ZyheedaCommands, Query<(Entity, &mut MovementPath)>) -> R
	where
		R: Default,
	{
		move |mut commands, mut query| {
			let Ok((entity, path)) = query.single_mut() else {
				return R::default();
			};

			commands
				.try_apply_on(&entity, |mut e| func(&mut e, path))
				.unwrap_or_default()
		}
	}

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	static SPEED: LazyLock<Speed> = LazyLock::new(|| Speed(UnitsPerSecond::from(42.)));

	mod current_movement {
		use super::*;

		#[test]
		fn target() {
			let path = MovementPath::target(Vec3::new(1., 2., 3.));

			assert_eq!(
				Some(MovementTarget::Point(Vec3::new(1., 2., 3.))),
				path.current_movement(),
			);
		}

		#[test]
		fn direction() {
			let path = MovementPath::direction(Dir3::X);

			assert_eq!(Some(MovementTarget::Dir(Dir3::X)), path.current_movement());
		}

		#[test]
		fn path() {
			let path = MovementPath::path([Vec3::new(1., 2., 3.), Vec3::new(3., 4., 5.)]);

			assert_eq!(
				Some(MovementTarget::Point(Vec3::new(3., 4., 5.))),
				path.current_movement(),
			);
		}

		#[test]
		fn stop() {
			let path = MovementPath::stop();

			assert_eq!(None, path.current_movement());
		}
	}

	mod path {
		use super::*;

		#[test]
		fn insert_movement_from_path() -> Result<(), RunSystemError> {
			let mut app = setup();
			let wp = Vec3::new(1., 2., 3.);
			let entity = app
				.world_mut()
				.spawn(MovementPath::path([wp, Vec3::default()]))
				.id();

			app.world_mut()
				.run_system_once(system(move |entity, components| {
					MovementPath::update(entity, components, *SPEED)
				}))?;

			assert_eq!(
				Some(&OngoingMovement::target(wp)),
				app.world().entity(entity).get::<OngoingMovement>(),
			);
			Ok(())
		}

		#[test]
		fn dequeue_path() -> Result<(), RunSystemError> {
			let mut app = setup();
			let other = Vec3::new(1., 2., 3.);
			let entity = app
				.world_mut()
				.spawn(MovementPath::path([Vec3::new(-1., -2., -3.), other]))
				.id();

			app.world_mut()
				.run_system_once(system(move |entity, components| {
					MovementPath::update(entity, components, *SPEED)
				}))?;

			assert_eq!(
				Some(&MovementPath::path([other])),
				app.world().entity(entity).get::<MovementPath>()
			);
			Ok(())
		}

		#[test]
		fn is_not_done_when_path_can_be_dequeued() -> Result<(), RunSystemError> {
			let mut app = setup();
			let wp = Vec3::new(1., 2., 3.);
			app.world_mut().spawn(MovementPath::path([wp]));

			let is_done = app
				.world_mut()
				.run_system_once(system(|entity, components| {
					MovementPath::update(entity, components, *SPEED)
				}))?;

			assert_eq!(Done::from(false), is_done);
			Ok(())
		}

		#[test]
		fn is_done_when_path_can_not_be_dequeued() -> Result<(), RunSystemError> {
			let mut app = setup();
			app.world_mut().spawn(MovementPath::path([]));

			let is_done = app
				.world_mut()
				.run_system_once(system(|entity, components| {
					MovementPath::update(entity, components, *SPEED)
				}))?;

			assert_eq!(Done::from(true), is_done);
			Ok(())
		}

		#[test]
		fn stop_movement_when_done() -> Result<(), RunSystemError> {
			let mut app = setup();
			let entity = app.world_mut().spawn(MovementPath::path([])).id();

			app.world_mut()
				.run_system_once(system(|entity, components| {
					MovementPath::update(entity, components, *SPEED)
				}))?;

			assert_eq!(
				Some(&OngoingMovement::Stopped),
				app.world().entity(entity).get::<OngoingMovement>()
			);
			Ok(())
		}

		#[test]
		fn set_stop_on_stop() -> Result<(), RunSystemError> {
			let mut app = setup();
			let entity = app.world_mut().spawn(MovementPath::path([])).id();

			app.world_mut().run_system_once(system(|entity, _| {
				<MovementPath as MovementUpdate>::stop(entity)
			}))?;

			assert_eq!(
				Some(&MovementPath::stop()),
				app.world().entity(entity).get::<MovementPath>(),
			);
			Ok(())
		}
	}

	mod wasd {
		use super::*;

		#[test]
		fn insert_movement_from_wasd() -> Result<(), RunSystemError> {
			let mut app = setup();
			let dir = Dir3::NEG_Z;
			let entity = app.world_mut().spawn(MovementPath::direction(dir)).id();

			app.world_mut()
				.run_system_once(system(move |entity, components| {
					MovementPath::update(entity, components, *SPEED)
				}))?;

			assert_eq!(
				Some(&OngoingMovement::target(dir)),
				app.world().entity(entity).get::<OngoingMovement>(),
			);
			Ok(())
		}

		#[test]
		fn is_not_done_when_wasd_has_some_target() -> Result<(), RunSystemError> {
			let mut app = setup();
			let dir = Dir3::NEG_Z;
			app.world_mut().spawn(MovementPath::direction(dir));

			let is_done = app
				.world_mut()
				.run_system_once(system(|entity, components| {
					MovementPath::update(entity, components, *SPEED)
				}))?;

			assert_eq!(Done::from(false), is_done);
			Ok(())
		}

		#[test]
		fn take_direction() -> Result<(), RunSystemError> {
			let mut app = setup();
			let dir = Dir3::NEG_Z;
			let entity = app.world_mut().spawn(MovementPath::direction(dir)).id();

			_ = app
				.world_mut()
				.run_system_once(system(|entity, components| {
					MovementPath::update(entity, components, *SPEED)
				}))?;

			assert_eq!(
				Some(&MovementPath::stop()),
				app.world().entity(entity).get::<MovementPath>(),
			);
			Ok(())
		}

		#[test]
		fn set_stop_on_stop() -> Result<(), RunSystemError> {
			let mut app = setup();
			let entity = app.world_mut().spawn(MovementPath::stop()).id();

			app.world_mut().run_system_once(system(|entity, _| {
				<MovementPath as MovementUpdate>::stop(entity)
			}))?;

			assert_eq!(
				Some(&MovementPath::stop()),
				app.world().entity(entity).get::<MovementPath>(),
			);
			Ok(())
		}
	}
}
