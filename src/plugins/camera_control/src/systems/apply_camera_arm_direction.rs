use crate::components::camera_arm::CameraArm;
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::traits::{
	accessors::get::GetContextMut,
	handles_graphics::{CameraHandle, CameraTransformMut},
};

impl CameraArm {
	pub(crate) fn apply_direction<TCamera>(
		mut camera: StaticSystemParam<TCamera>,
		arms: Query<(&Transform, &CameraArm)>,
	) where
		TCamera: for<'c> GetContextMut<CameraHandle, TContext<'c>: CameraTransformMut>,
	{
		let mut camera = TCamera::get_context_mut(&mut camera, CameraHandle);

		for (transform, arm) in arms {
			*camera.camera_transform_mut() =
				Transform::from_translation(transform.translation + arm.direction * *arm.distance)
					.looking_at(transform.translation, Vec3::Y);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{tools::Units, traits::handles_graphics::CameraTransform};
	use testing::SingleThreadedApp;

	#[derive(Resource, Debug, PartialEq, Default)]
	struct _Camera {
		transform: Transform,
	}

	impl CameraTransform for &mut _Camera {
		fn camera_transform(&self) -> &'_ Transform {
			&self.transform
		}
	}

	impl CameraTransformMut for &mut _Camera {
		fn camera_transform_mut(&mut self) -> &'_ mut Transform {
			&mut self.transform
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<_Camera>();
		app.add_systems(Update, CameraArm::apply_direction::<ResMut<_Camera>>);

		app
	}

	#[test]
	fn apply_offset() {
		let mut app = setup();
		app.world_mut().spawn(CameraArm {
			direction: Dir3::NEG_Z,
			distance: Units::from(10.),
			..default()
		});

		app.update();

		assert_eq!(
			&_Camera {
				transform: Transform::from_xyz(0., 0., -10.).looking_at(Vec3::ZERO, Vec3::Y)
			},
			app.world().resource::<_Camera>(),
		);
	}

	#[test]
	fn apply_offset_based_on_arm_center() {
		let mut app = setup();
		app.world_mut().spawn((
			Transform::from_xyz(3., 4., 5.),
			CameraArm {
				direction: Dir3::NEG_Z,
				distance: Units::from(10.),
				..default()
			},
		));

		app.update();

		assert_eq!(
			&_Camera {
				transform: Transform::from_xyz(3., 4., -5.)
					.looking_at(Vec3::new(3., 4., 5.), Vec3::Y)
			},
			app.world().resource::<_Camera>(),
		);
	}
}
