use crate::{
	components::camera_labels::MovableCamera,
	resources::camera_parameters::CameraParameters,
};
use bevy::prelude::*;
use common::{traits::accessors::get::TryApplyOn, zyheeda_commands::ZyheedaCommands};

impl CameraParameters {
	pub(crate) fn apply_changes(
		mut commands: ZyheedaCommands,
		mut world_camera: ResMut<Self>,
		cameras: Query<Entity, With<MovableCamera>>,
	) {
		if !world_camera.is_changed() {
			return;
		}

		for entity in cameras {
			commands.try_apply_on(&entity, |mut e| {
				e.try_insert(world_camera.transform);
			});
		}

		let Some(ui_cam) = world_camera.ui_cam else {
			return;
		};

		for ui in world_camera.uis.drain(..) {
			commands.try_apply_on(&ui, |mut ui| {
				ui.try_insert(UiTargetCamera(ui_cam));
			});
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use testing::{SingleThreadedApp, fake_entity};

	fn setup(world_camera: CameraParameters) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(world_camera);
		app.add_systems(Update, CameraParameters::apply_changes);

		app
	}

	#[test]
	fn apply_transform() {
		let mut app = setup(CameraParameters {
			transform: Transform::from_xyz(1., 2., 3.),
			..default()
		});
		let entity = app.world_mut().spawn(MovableCamera).id();

		app.update();

		assert_eq!(
			Some(&Transform::from_xyz(1., 2., 3.)),
			app.world().entity(entity).get::<Transform>(),
		);
	}

	#[test]
	fn do_not_apply_transform_when_not_movable_camera() {
		let mut app = setup(CameraParameters {
			transform: Transform::from_xyz(1., 2., 3.),
			..default()
		});
		let entity = app.world_mut().spawn_empty().id();

		app.update();

		assert_eq!(None, app.world().entity(entity).get::<Transform>());
	}

	#[test]
	fn insert_ui_target_camera() {
		let mut app = setup(CameraParameters {
			ui_cam: Some(fake_entity!(42)),
			..default()
		});
		let entity = app.world_mut().spawn_empty().id();
		app.world_mut().resource_mut::<CameraParameters>().uis = vec![entity];

		app.update();

		assert_eq!(
			Some(&UiTargetCamera(fake_entity!(42))),
			app.world().entity(entity).get::<UiTargetCamera>(),
		);
	}

	#[test]
	fn clear_uis() {
		let mut app = setup(CameraParameters {
			ui_cam: Some(fake_entity!(42)),
			..default()
		});
		let entity = app.world_mut().spawn_empty().id();
		app.world_mut().resource_mut::<CameraParameters>().uis = vec![entity];

		app.update();

		assert!(app.world().resource::<CameraParameters>().uis.is_empty());
	}

	#[test]
	fn act_only_once() {
		let mut app = setup(CameraParameters {
			ui_cam: Some(fake_entity!(42)),
			..default()
		});
		let ui = app.world_mut().spawn_empty().id();
		app.world_mut().resource_mut::<CameraParameters>().uis = vec![ui];
		let cam = app.world_mut().spawn(MovableCamera).id();

		app.update();
		app.world_mut().entity_mut(cam).remove::<Transform>();
		app.world_mut().entity_mut(ui).remove::<UiTargetCamera>();
		app.update();

		assert_eq!(
			(None, None),
			(
				app.world().entity(cam).get::<Transform>(),
				app.world().entity(ui).get::<UiTargetCamera>(),
			)
		);
	}

	#[test]
	fn act_again_if_world_camera_changed() {
		let mut app = setup(CameraParameters {
			ui_cam: Some(fake_entity!(42)),
			transform: Transform::from_xyz(1., 2., 3.),
			..default()
		});
		let ui = app.world_mut().spawn_empty().id();
		app.world_mut().resource_mut::<CameraParameters>().uis = vec![ui];
		let cam = app.world_mut().spawn(MovableCamera).id();

		app.update();
		app.world_mut().entity_mut(cam).remove::<Transform>();
		app.world_mut().entity_mut(ui).remove::<UiTargetCamera>();
		app.world_mut().resource_mut::<CameraParameters>().uis = vec![ui];
		app.update();

		assert_eq!(
			(
				Some(&Transform::from_xyz(1., 2., 3.)),
				Some(&UiTargetCamera(fake_entity!(42))),
			),
			(
				app.world().entity(cam).get::<Transform>(),
				app.world().entity(ui).get::<UiTargetCamera>(),
			)
		);
	}
}
