use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use common::{
	errors::{ErrorData, Level},
	traits::handles_physics::{NoWorldCameraSet, SetWorldCamera},
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(
	Component,
	SavableComponent,
	Debug,
	PartialEq,
	Eq,
	Hash,
	Default,
	Clone,
	Copy,
	Serialize,
	Deserialize,
)]
#[savable_component(id = "player camera")]
pub struct PlayerCamera;

impl PlayerCamera {
	pub(crate) fn set_as_world_camera<TPhysics>(
		on_add: On<Add, Self>,
		physics: StaticSystemParam<TPhysics>,
	) -> Result<(), WorldCameraAlreadySet>
	where
		TPhysics: for<'w, 's> SystemParam<Item<'w, 's>: NoWorldCameraSet>,
	{
		let Some(setter) = physics.into_inner().no_world_camera_set() else {
			return Err(WorldCameraAlreadySet);
		};

		setter.set_world_camera(on_add.entity);
		Ok(())
	}
}

#[derive(Debug, PartialEq)]
pub(crate) struct WorldCameraAlreadySet;

impl Display for WorldCameraAlreadySet {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "World camera is already set")
	}
}

impl ErrorData for WorldCameraAlreadySet {
	fn level(&self) -> common::errors::Level {
		Level::Error
	}

	fn label() -> impl Display {
		"World Camera Already Set Error"
	}

	fn into_details(self) -> impl Display {
		self
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use macros::simple_mock;
	use mockall::predicate::eq;
	use testing::{Mock, SingleThreadedApp};

	#[derive(Resource)]
	enum _Setter {
		NoCamSet(Mock_Setter),
		CamAlreadySet,
	}

	#[derive(SystemParam)]
	struct _Param<'w> {
		setter: ResMut<'w, _Setter>,
	}

	impl<'w> NoWorldCameraSet for _Param<'w> {
		type TSetter = Mock_Setter;

		fn no_world_camera_set(mut self) -> Option<Self::TSetter> {
			let mut return_setter = _Setter::CamAlreadySet;

			std::mem::swap(&mut return_setter, &mut self.setter);

			match return_setter {
				_Setter::NoCamSet(mock_setter) => Some(mock_setter),
				_Setter::CamAlreadySet => None,
			}
		}
	}

	simple_mock! {
		_Setter {}
		impl SetWorldCamera for _Setter {
			fn set_world_camera(self, entity: Entity);
		}
	}

	#[derive(Resource, Debug, PartialEq)]
	struct _Result(Result<(), WorldCameraAlreadySet>);

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(PlayerCamera::set_as_world_camera::<_Param>.pipe(
			|In(r), mut c: Commands| {
				c.insert_resource(_Result(r));
			},
		));

		app
	}

	#[test]
	fn set_camera() {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();
		app.insert_resource(_Setter::NoCamSet(Mock_Setter::new_mock(assert_set_to(
			entity,
		))));

		app.world_mut().entity_mut(entity).insert(PlayerCamera);

		fn assert_set_to(entity: Entity) -> impl FnMut(&mut Mock_Setter) {
			move |mock| {
				mock.expect_set_world_camera()
					.times(1)
					.with(eq(entity))
					.return_const(());
			}
		}
	}

	#[test]
	fn return_ok() {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();
		app.insert_resource(_Setter::NoCamSet(Mock_Setter::new_mock(|mock| {
			mock.expect_set_world_camera().return_const(());
		})));

		app.world_mut().entity_mut(entity).insert(PlayerCamera);

		assert_eq!(&_Result(Ok(())), app.world().resource::<_Result>());
	}

	#[test]
	fn return_already_set_error() {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();
		app.insert_resource(_Setter::CamAlreadySet);

		app.world_mut().entity_mut(entity).insert(PlayerCamera);

		assert_eq!(
			&_Result(Err(WorldCameraAlreadySet)),
			app.world().resource::<_Result>(),
		);
	}
}
