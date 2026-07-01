use crate::{
	components::camera_labels::{
		AgentsPass,
		CompositePass,
		EffectLightPass,
		OutlinePass,
		UiPass,
		VisibilityPass,
		WorldLight,
		WorldPass,
	},
	resources::camera_parameters::CameraParameters,
};
use bevy::{ecs::entity::EntityHashSet, prelude::*};
use common::{
	errors::{ErrorData, Level},
	zyheeda_commands::ZyheedaCommands,
};
use std::fmt::Display;

impl UiPass {
	pub(crate) fn process_new_ui_pass(
		new_ui_passes: Query<(Entity, &Transform), Added<Self>>,
		mut camera_parameters: ResMut<CameraParameters>,
		mut commands: ZyheedaCommands,
		mut despawned: RemovedComponents<Self>,
	) -> Result<(), Vec<UiPassDuplicateError>> {
		for despawned in despawned.read() {
			if !matches!(camera_parameters.ui_cam, Some(ui_cam) if ui_cam == despawned ) {
				continue;
			}

			camera_parameters.transform = Transform::default();
			camera_parameters.ui_cam = None;
		}

		let duplicates = new_ui_passes
			.into_iter()
			.filter_map(|(entity, transform)| match camera_parameters.ui_cam {
				Some(ui_cam) => Some(UiPassDuplicateError { entity, ui_cam }),
				None => {
					camera_parameters.transform = *transform;
					camera_parameters.ui_cam = Some(entity);

					commands.spawn((BondedTo(entity), WorldPass, *transform));
					commands.spawn((BondedTo(entity), AgentsPass, *transform));
					commands.spawn((BondedTo(entity), VisibilityPass, *transform));
					commands.spawn((BondedTo(entity), EffectLightPass, *transform));
					commands.spawn((BondedTo(entity), OutlinePass, *transform));
					commands.spawn((BondedTo(entity), CompositePass, *transform));
					commands.spawn((BondedTo(entity), WorldLight, *transform));

					None
				}
			})
			.collect::<Vec<_>>();

		if !duplicates.is_empty() {
			return Err(duplicates);
		}

		Ok(())
	}
}

#[derive(Component)]
#[relationship_target(relationship = BondedTo, linked_spawn)]
struct Bonds(EntityHashSet);

#[derive(Component)]
#[relationship(relationship_target = Bonds)]
struct BondedTo(Entity);

#[derive(Debug, PartialEq)]
pub(crate) struct UiPassDuplicateError {
	entity: Entity,
	ui_cam: Entity,
}

impl Display for UiPassDuplicateError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let entity = self.entity;
		let ui_cam = self.ui_cam;

		write!(
			f,
			"A `UiPass` was spawned on {entity}, but it already exists on {ui_cam}",
		)
	}
}

impl ErrorData for UiPassDuplicateError {
	fn level(&self) -> Level {
		Level::Error
	}

	fn label() -> impl Display {
		"UI Pass Duplicate Error"
	}

	fn into_details(self) -> impl Display {
		self
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::resources::camera_parameters::CameraParameters;
	use std::fmt::Debug;
	use test_case::test_case;
	use testing::{SingleThreadedApp, assert_count};

	#[derive(Resource, Debug, PartialEq)]
	struct _Result(Result<(), Vec<UiPassDuplicateError>>);

	#[derive(Resource, Debug, PartialEq)]
	struct _CameraParametersChanged(bool);

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<CameraParameters>();
		app.add_systems(
			Update,
			(
				UiPass::process_new_ui_pass.pipe(|In(r), mut c: Commands| {
					c.insert_resource(_Result(r));
				}),
				|camera_parameters: Res<CameraParameters>, mut c: Commands| {
					c.insert_resource(_CameraParametersChanged(camera_parameters.is_changed()));
				},
			)
				.chain(),
		);

		app
	}

	#[test]
	fn update_camera_parameters() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((UiPass, Transform::from_xyz(1., 2., 3.)))
			.id();

		app.update();

		assert_eq!(
			(
				&CameraParameters {
					ui_cam: Some(entity),
					transform: Transform::from_xyz(1., 2., 3.),
					..default()
				},
				&_Result(Ok(()))
			),
			(
				app.world().resource::<CameraParameters>(),
				app.world().resource::<_Result>(),
			)
		);
	}

	#[test_case(WorldPass; "world")]
	#[test_case(AgentsPass; "agents")]
	#[test_case(CompositePass; "composite")]
	#[test_case(EffectLightPass; "effect light")]
	#[test_case(OutlinePass; "outline")]
	#[test_case(VisibilityPass; "visibility")]
	#[test_case(WorldLight; "world light")]
	fn spawn_other_camera<T>(other_camera: T)
	where
		T: Component + Debug + PartialEq,
	{
		let mut app = setup();
		app.world_mut()
			.spawn((UiPass, Transform::from_xyz(1., 2., 3.)));

		app.update();

		let mut query = app.world_mut().query::<(&T, &Transform)>();
		let [(cam, transform)] = assert_count!(1, query.iter(app.world()));
		assert_eq!(
			(
				&other_camera,
				&Transform::from_xyz(1., 2., 3.),
				&_Result(Ok(()))
			),
			(cam, transform, app.world().resource::<_Result>(),)
		);
	}

	#[test_case(WorldPass; "world")]
	#[test_case(AgentsPass; "agents")]
	#[test_case(CompositePass; "composite")]
	#[test_case(EffectLightPass; "effect light")]
	#[test_case(OutlinePass; "outline")]
	#[test_case(VisibilityPass; "visibility")]
	#[test_case(WorldLight; "world light")]
	fn despawn_other_camera<T>(_: T)
	where
		T: Component + Debug + PartialEq,
	{
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((UiPass, Transform::from_xyz(1., 2., 3.)))
			.id();

		app.update();
		app.world_mut().entity_mut(entity).despawn();
		app.update();

		let mut query = app.world_mut().query::<(&T, &Transform)>();
		assert_count!(0, query.iter(app.world()));
	}

	#[test]
	fn despawn_camera() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((UiPass, Transform::from_xyz(1., 2., 3.)))
			.id();

		app.update();
		app.world_mut().entity_mut(entity).despawn();
		app.update();

		assert_eq!(
			(&CameraParameters::default(), &_Result(Ok(()))),
			(
				app.world().resource::<CameraParameters>(),
				app.world().resource::<_Result>(),
			)
		);
	}

	#[test]
	fn ignore_and_report_duplicate_ui_cams_in_one_frame() {
		let mut app = setup();
		let ui_cam = app
			.world_mut()
			.spawn((UiPass, Transform::from_xyz(1., 2., 3.)))
			.id();
		let duplicate_a = app
			.world_mut()
			.spawn((UiPass, Transform::from_xyz(3., 4., 5.)))
			.id();
		let duplicate_b = app
			.world_mut()
			.spawn((UiPass, Transform::from_xyz(5., 4., 3.)))
			.id();

		app.update();

		assert_eq!(
			(
				&CameraParameters {
					ui_cam: Some(ui_cam),
					transform: Transform::from_xyz(1., 2., 3.),
					..default()
				},
				&_Result(Err(vec![
					UiPassDuplicateError {
						ui_cam,
						entity: duplicate_a
					},
					UiPassDuplicateError {
						ui_cam,
						entity: duplicate_b
					}
				]))
			),
			(
				app.world().resource::<CameraParameters>(),
				app.world().resource::<_Result>(),
			)
		);
	}

	#[test]
	fn ignore_and_report_duplicate_ui_cams_across_frames() {
		let mut app = setup();
		let ui_cam = app
			.world_mut()
			.spawn((UiPass, Transform::from_xyz(1., 2., 3.)))
			.id();

		app.update();
		let duplicate_a = app
			.world_mut()
			.spawn((UiPass, Transform::from_xyz(3., 4., 5.)))
			.id();
		let duplicate_b = app
			.world_mut()
			.spawn((UiPass, Transform::from_xyz(5., 4., 3.)))
			.id();
		app.update();

		assert_eq!(
			(
				&CameraParameters {
					ui_cam: Some(ui_cam),
					transform: Transform::from_xyz(1., 2., 3.),
					..default()
				},
				&_Result(Err(vec![
					UiPassDuplicateError {
						ui_cam,
						entity: duplicate_a
					},
					UiPassDuplicateError {
						ui_cam,
						entity: duplicate_b
					}
				]))
			),
			(
				app.world().resource::<CameraParameters>(),
				app.world().resource::<_Result>(),
			)
		);
	}

	#[test]
	fn allow_new_ui_pass_if_old_is_despawned() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((UiPass, Transform::from_xyz(3., 4., 5.)))
			.id();

		app.update();
		app.world_mut().entity_mut(entity).despawn();
		app.update();
		let entity = app
			.world_mut()
			.spawn((UiPass, Transform::from_xyz(1., 2., 3.)))
			.id();
		app.update();

		assert_eq!(
			(
				&CameraParameters {
					ui_cam: Some(entity),
					transform: Transform::from_xyz(1., 2., 3.),
					..default()
				},
				&_Result(Ok(()))
			),
			(
				app.world().resource::<CameraParameters>(),
				app.world().resource::<_Result>(),
			)
		);
	}

	#[test]
	fn do_not_allow_new_ui_pass_if_duplicate_is_despawned() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((UiPass, Transform::from_xyz(1., 2., 3.)))
			.id();
		let duplicate = app
			.world_mut()
			.spawn((UiPass, Transform::from_xyz(3., 4., 5.)))
			.id();

		app.update();
		app.world_mut().entity_mut(duplicate).despawn();
		app.update();
		let duplicate = app
			.world_mut()
			.spawn((UiPass, Transform::from_xyz(5., 4., 3.)))
			.id();
		app.update();

		assert_eq!(
			(
				&CameraParameters {
					ui_cam: Some(entity),
					transform: Transform::from_xyz(1., 2., 3.),
					..default()
				},
				&_Result(Err(vec![UiPassDuplicateError {
					entity: duplicate,
					ui_cam: entity
				}]))
			),
			(
				app.world().resource::<CameraParameters>(),
				app.world().resource::<_Result>(),
			)
		);
	}

	#[test]
	fn ignore_when_ui_pass_missing() {
		let mut app = setup();
		app.world_mut().spawn(Transform::from_xyz(1., 2., 3.));

		app.update();

		assert_eq!(
			(&CameraParameters::default(), &_Result(Ok(()))),
			(
				app.world().resource::<CameraParameters>(),
				app.world().resource::<_Result>(),
			)
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();
		app.world_mut()
			.spawn((UiPass, Transform::from_xyz(1., 2., 3.)));

		app.update();
		app.update();

		assert_eq!(
			(&_CameraParametersChanged(false), &_Result(Ok(()))),
			(
				app.world().resource::<_CameraParametersChanged>(),
				app.world().resource::<_Result>(),
			)
		);
	}
}
