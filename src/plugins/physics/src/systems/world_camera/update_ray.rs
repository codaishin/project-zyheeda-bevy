use std::fmt::Display;

use crate::components::world_camera::WorldCamera;
use bevy::{ecs::query::QuerySingleError, prelude::*};
use common::{
	errors::{ErrorData, Level},
	traits::get_ray::GetCamRay,
};

impl WorldCamera {
	pub(crate) fn update_ray(
		cameras: Query<(&mut Self, Ref<Camera>, Ref<GlobalTransform>)>,
		windows: Query<Ref<Window>>,
	) -> Result<(), WindowError> {
		Self::update_ray_internal(cameras, windows)
	}

	fn update_ray_internal<TCamera, TWindow>(
		mut cameras: Query<(&mut Self, Ref<TCamera>, Ref<GlobalTransform>)>,
		windows: Query<Ref<TWindow>>,
	) -> Result<(), WindowError>
	where
		TCamera: Component + GetCamRay<TWindow>,
		TWindow: Component,
	{
		let window = match windows.single() {
			Err(QuerySingleError::NoEntities(..)) => return Err(WindowError::None),
			Err(QuerySingleError::MultipleEntities(..)) => return Err(WindowError::Multiple),
			Ok(window) => window,
		};
		let window_is_changed = window.is_changed();

		for (mut world_camera, camera, transform) in &mut cameras {
			if !window_is_changed && !camera.is_changed() && !transform.is_changed() {
				continue;
			}

			world_camera.ray = camera.get_ray(&transform, &window);
		}

		Ok(())
	}
}

#[derive(Debug, PartialEq)]
pub(crate) enum WindowError {
	None,
	Multiple,
}

impl ErrorData for WindowError {
	fn level(&self) -> Level {
		Level::Error
	}

	fn label() -> impl Display {
		"Window Error"
	}

	fn into_details(self) -> impl Display {
		let reason = match self {
			WindowError::None => "No window found",
			WindowError::Multiple => "Multiple windows found",
		};

		format!("Cannot determine camera ray. {reason}")
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Component, NestedMocks)]
	struct _Camera {
		mock: Mock_Camera,
	}

	#[automock]
	impl GetCamRay<_Window> for _Camera {
		fn get_ray(&self, camera_transform: &GlobalTransform, window: &_Window) -> Option<Ray3d> {
			self.mock.get_ray(camera_transform, window)
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Window;

	#[derive(Resource, Debug, PartialEq)]
	struct _Result(Result<(), WindowError>);

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			WorldCamera::update_ray_internal::<_Camera, _Window>.pipe(
				|In(result), mut commands: Commands| {
					commands.insert_resource(_Result(result));
				},
			),
		);

		app
	}

	#[test]
	fn update_ray() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				WorldCamera::default(),
				_Camera::new().with_mock(|mock| {
					mock.expect_get_ray()
						.times(1)
						.with(eq(GlobalTransform::from_xyz(1., 2., 3.)), eq(_Window))
						.return_const(Ray3d {
							origin: Vec3::new(4., 5., 6.),
							direction: Dir3::NEG_X,
						});
				}),
				GlobalTransform::from_xyz(1., 2., 3.),
			))
			.id();
		app.world_mut().spawn(_Window);

		app.update();

		assert_eq!(
			Some(&WorldCamera {
				ray: Some(Ray3d {
					origin: Vec3::new(4., 5., 6.),
					direction: Dir3::NEG_X,
				}),
				..default()
			}),
			app.world().entity(entity).get::<WorldCamera>()
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				WorldCamera::default(),
				_Camera::new().with_mock(|mock| {
					mock.expect_get_ray().return_const(Ray3d {
						origin: Vec3::new(4., 5., 6.),
						direction: Dir3::NEG_X,
					});
				}),
				GlobalTransform::from_xyz(1., 2., 3.),
			))
			.id();
		app.world_mut().spawn(_Window);

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.insert(WorldCamera::default());
		app.update();

		assert_eq!(
			Some(&WorldCamera::default()),
			app.world().entity(entity).get::<WorldCamera>()
		);
	}

	#[test]
	fn act_again_if_transform_changed() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				WorldCamera::default(),
				_Camera::new().with_mock(|mock| {
					mock.expect_get_ray().return_const(Ray3d {
						origin: Vec3::new(4., 5., 6.),
						direction: Dir3::NEG_X,
					});
				}),
				GlobalTransform::from_xyz(1., 2., 3.),
			))
			.id();
		app.world_mut().spawn(_Window);

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.insert(WorldCamera::default());
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<GlobalTransform>()
			.as_deref_mut();
		app.update();

		assert_eq!(
			Some(&WorldCamera {
				ray: Some(Ray3d {
					origin: Vec3::new(4., 5., 6.),
					direction: Dir3::NEG_X,
				}),
				..default()
			}),
			app.world().entity(entity).get::<WorldCamera>()
		);
	}

	#[test]
	fn act_again_if_window_changed() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				WorldCamera::default(),
				_Camera::new().with_mock(|mock| {
					mock.expect_get_ray().return_const(Ray3d {
						origin: Vec3::new(4., 5., 6.),
						direction: Dir3::NEG_X,
					});
				}),
				GlobalTransform::from_xyz(1., 2., 3.),
			))
			.id();
		let window = app.world_mut().spawn(_Window).id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.insert(WorldCamera::default());
		app.world_mut()
			.entity_mut(window)
			.get_mut::<_Window>()
			.as_deref_mut();
		app.update();

		assert_eq!(
			Some(&WorldCamera {
				ray: Some(Ray3d {
					origin: Vec3::new(4., 5., 6.),
					direction: Dir3::NEG_X,
				}),
				..default()
			}),
			app.world().entity(entity).get::<WorldCamera>()
		);
	}

	#[test]
	fn act_again_if_camera_changed() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				WorldCamera::default(),
				_Camera::new().with_mock(|mock| {
					mock.expect_get_ray().return_const(Ray3d {
						origin: Vec3::new(4., 5., 6.),
						direction: Dir3::NEG_X,
					});
				}),
				GlobalTransform::from_xyz(1., 2., 3.),
			))
			.id();
		app.world_mut().spawn(_Window);

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.insert(WorldCamera::default());
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<_Camera>()
			.as_deref_mut();
		app.update();

		assert_eq!(
			Some(&WorldCamera {
				ray: Some(Ray3d {
					origin: Vec3::new(4., 5., 6.),
					direction: Dir3::NEG_X,
				}),
				..default()
			}),
			app.world().entity(entity).get::<WorldCamera>()
		);
	}

	#[test]
	fn missing_window_error() {
		let mut app = setup();
		app.world_mut().spawn((
			WorldCamera::default(),
			_Camera::new().with_mock(|mock| {
				mock.expect_get_ray().return_const(None);
			}),
			GlobalTransform::from_xyz(1., 2., 3.),
		));

		app.update();

		assert_eq!(
			&_Result(Err(WindowError::None)),
			app.world().resource::<_Result>(),
		);
	}

	#[test]
	fn multiple_windows_error() {
		let mut app = setup();
		app.world_mut().spawn((
			WorldCamera::default(),
			_Camera::new().with_mock(|mock| {
				mock.expect_get_ray().return_const(None);
			}),
			GlobalTransform::from_xyz(1., 2., 3.),
		));
		app.world_mut().spawn(_Window);
		app.world_mut().spawn(_Window);

		app.update();

		assert_eq!(
			&_Result(Err(WindowError::Multiple)),
			app.world().resource::<_Result>(),
		);
	}
}
