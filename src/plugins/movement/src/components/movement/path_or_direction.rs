use crate::{
	components::new_movement::{IsMoving, NewMovement},
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
#[savable_component(id = "movement")]
pub struct PathOrDirection {
	pub(crate) mode: Mode,
}

impl PathOrDirection {
	fn take_next(&mut self) -> Option<MovementTarget> {
		match &mut self.mode {
			Mode::PathTarget(_) => None,
			Mode::Direction(target) => {
				let wp = Some(MovementTarget::Dir(*target));
				self.mode = Mode::PathTarget(None);
				wp
			}
			Mode::Path(path) => path.pop_front().map(MovementTarget::Point),
		}
	}

	pub(crate) fn stop() -> Self {
		Self {
			mode: Mode::PathTarget(None),
		}
	}

	pub(crate) fn direction(dir: Dir3) -> Self {
		Self {
			mode: Mode::Direction(dir),
		}
	}

	pub(crate) fn target(pos: Vec3) -> Self {
		Self {
			mode: Mode::PathTarget(Some(pos)),
		}
	}

	pub(crate) fn path(path: impl Into<VecDeque<Vec3>>) -> Self {
		Self {
			mode: Mode::Path(path.into()),
		}
	}
}

impl<T> From<T> for PathOrDirection
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

impl MovementUpdate for PathOrDirection {
	type TComponents = &'static mut Self;
	type TConstraint = Without<IsMoving>;

	fn update(entity: &mut ZyheedaEntityCommands, mut path: Mut<Self>, _: Speed) -> Done {
		match path.take_next() {
			Some(next) => {
				entity.try_insert(NewMovement::to(next));
				Done(false)
			}
			None => {
				entity.try_insert(NewMovement::Stopped);
				Done(true)
			}
		}
	}

	fn stop(entity: &mut ZyheedaEntityCommands) {
		entity.try_insert(Self::stop());
	}
}

impl CurrentMovement for PathOrDirection {
	fn current_movement(&self) -> Option<MovementTarget> {
		match &self.mode {
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
		func: impl Fn(&mut ZyheedaEntityCommands, Mut<PathOrDirection>) -> R,
	) -> impl Fn(ZyheedaCommands, Query<(Entity, &mut PathOrDirection)>) -> R
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
			let path = PathOrDirection::target(Vec3::new(1., 2., 3.));

			assert_eq!(
				Some(MovementTarget::Point(Vec3::new(1., 2., 3.))),
				path.current_movement(),
			);
		}

		#[test]
		fn direction() {
			let path = PathOrDirection::direction(Dir3::X);

			assert_eq!(Some(MovementTarget::Dir(Dir3::X)), path.current_movement());
		}

		#[test]
		fn path() {
			let path = PathOrDirection::path([Vec3::new(1., 2., 3.), Vec3::new(3., 4., 5.)]);

			assert_eq!(
				Some(MovementTarget::Point(Vec3::new(3., 4., 5.))),
				path.current_movement(),
			);
		}

		#[test]
		fn stop() {
			let path = PathOrDirection::stop();

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
				.spawn(PathOrDirection {
					mode: Mode::Path(VecDeque::from([wp, Vec3::default()])),
				})
				.id();

			app.world_mut()
				.run_system_once(system(move |entity, components| {
					PathOrDirection::update(entity, components, *SPEED)
				}))?;

			assert_eq!(
				Some(&NewMovement::to(wp)),
				app.world().entity(entity).get::<NewMovement>(),
			);
			Ok(())
		}

		#[test]
		fn dequeue_path() -> Result<(), RunSystemError> {
			let mut app = setup();
			let other = Vec3::new(1., 2., 3.);
			let entity = app
				.world_mut()
				.spawn(PathOrDirection {
					mode: Mode::Path(VecDeque::from([Vec3::new(-1., -2., -3.), other])),
				})
				.id();

			app.world_mut()
				.run_system_once(system(move |entity, components| {
					PathOrDirection::update(entity, components, *SPEED)
				}))?;

			assert_eq!(
				Some(&PathOrDirection {
					mode: Mode::Path(VecDeque::from([other])),
				}),
				app.world().entity(entity).get::<PathOrDirection>()
			);
			Ok(())
		}

		#[test]
		fn is_not_done_when_path_can_be_dequeued() -> Result<(), RunSystemError> {
			let mut app = setup();
			let wp = Vec3::new(1., 2., 3.);
			app.world_mut().spawn(PathOrDirection {
				mode: Mode::Path(VecDeque::from([wp])),
			});

			let is_done = app
				.world_mut()
				.run_system_once(system(|entity, components| {
					PathOrDirection::update(entity, components, *SPEED)
				}))?;

			assert_eq!(Done::from(false), is_done);
			Ok(())
		}

		#[test]
		fn is_done_when_path_can_not_be_dequeued() -> Result<(), RunSystemError> {
			let mut app = setup();
			app.world_mut().spawn(PathOrDirection {
				mode: Mode::Path(VecDeque::from([])),
			});

			let is_done = app
				.world_mut()
				.run_system_once(system(|entity, components| {
					PathOrDirection::update(entity, components, *SPEED)
				}))?;

			assert_eq!(Done::from(true), is_done);
			Ok(())
		}

		#[test]
		fn stop_movement_when_done() -> Result<(), RunSystemError> {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn(PathOrDirection {
					mode: Mode::Path(VecDeque::from([])),
				})
				.id();

			app.world_mut()
				.run_system_once(system(|entity, components| {
					PathOrDirection::update(entity, components, *SPEED)
				}))?;

			assert_eq!(
				Some(&NewMovement::Stopped),
				app.world().entity(entity).get::<NewMovement>()
			);
			Ok(())
		}

		#[test]
		fn set_stop_on_stop() -> Result<(), RunSystemError> {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn(PathOrDirection {
					mode: Mode::Path(VecDeque::from([])),
				})
				.id();

			app.world_mut().run_system_once(system(|entity, _| {
				<PathOrDirection as MovementUpdate>::stop(entity)
			}))?;

			assert_eq!(
				Some(&PathOrDirection::stop()),
				app.world().entity(entity).get::<PathOrDirection>(),
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
			let entity = app.world_mut().spawn(PathOrDirection::direction(dir)).id();

			app.world_mut()
				.run_system_once(system(move |entity, components| {
					PathOrDirection::update(entity, components, *SPEED)
				}))?;

			assert_eq!(
				Some(&NewMovement::to(dir)),
				app.world().entity(entity).get::<NewMovement>(),
			);
			Ok(())
		}

		#[test]
		fn is_not_done_when_wasd_has_some_target() -> Result<(), RunSystemError> {
			let mut app = setup();
			let dir = Dir3::NEG_Z;
			app.world_mut().spawn(PathOrDirection::direction(dir));

			let is_done = app
				.world_mut()
				.run_system_once(system(|entity, components| {
					PathOrDirection::update(entity, components, *SPEED)
				}))?;

			assert_eq!(Done::from(false), is_done);
			Ok(())
		}

		#[test]
		fn take_direction() -> Result<(), RunSystemError> {
			let mut app = setup();
			let dir = Dir3::NEG_Z;
			let entity = app.world_mut().spawn(PathOrDirection::direction(dir)).id();

			_ = app
				.world_mut()
				.run_system_once(system(|entity, components| {
					PathOrDirection::update(entity, components, *SPEED)
				}))?;

			assert_eq!(
				Some(&PathOrDirection::stop()),
				app.world().entity(entity).get::<PathOrDirection>(),
			);
			Ok(())
		}

		#[test]
		fn set_stop_on_stop() -> Result<(), RunSystemError> {
			let mut app = setup();
			let entity = app.world_mut().spawn(PathOrDirection::stop()).id();

			app.world_mut().run_system_once(system(|entity, _| {
				<PathOrDirection as MovementUpdate>::stop(entity)
			}))?;

			assert_eq!(
				Some(&PathOrDirection::stop()),
				app.world().entity(entity).get::<PathOrDirection>(),
			);
			Ok(())
		}
	}
}
