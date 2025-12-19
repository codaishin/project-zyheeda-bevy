use crate::components::{active_beam::ActiveBeam, skill_transform::SkillTransforms};
use bevy::prelude::*;

impl ActiveBeam {
	pub(crate) fn update_transform(
		beams: Query<(&ActiveBeam, &SkillTransforms), Changed<ActiveBeam>>,
		mut transforms: Query<&mut Transform>,
	) {
		for (beam, skill_transforms) in &beams {
			for skill_transform in skill_transforms.iter() {
				let Ok(mut transform) = transforms.get_mut(skill_transform) else {
					continue;
				};
				transform.translation.z = -*beam.length / 2.;
				transform.scale.y = *beam.length;
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::skill_transform::SkillTransformOf;
	use common::tools::Units;
	use testing::{IsChanged, SingleThreadedApp};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			(ActiveBeam::update_transform, IsChanged::<Transform>::detect).chain(),
		);

		app
	}

	#[test]
	fn update_z_of_direct_children() {
		let mut app = setup();
		let beam = app
			.world_mut()
			.spawn(ActiveBeam {
				length: Units::from(42.),
			})
			.id();
		let children = [
			app.world_mut()
				.spawn((Transform::default(), SkillTransformOf(beam)))
				.id(),
			app.world_mut()
				.spawn((Transform::default(), SkillTransformOf(beam)))
				.id(),
			app.world_mut()
				.spawn((Transform::default(), SkillTransformOf(beam)))
				.id(),
		];

		app.update();

		assert_eq!(
			[
				Some(&Transform::from_xyz(0., 0., -21.).with_scale(Vec3::ONE.with_y(42.))),
				Some(&Transform::from_xyz(0., 0., -21.).with_scale(Vec3::ONE.with_y(42.))),
				Some(&Transform::from_xyz(0., 0., -21.).with_scale(Vec3::ONE.with_y(42.))),
			],
			app.world().entity(children).map(|e| e.get::<Transform>()),
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();
		let beam = app
			.world_mut()
			.spawn(ActiveBeam {
				length: Units::from(42.),
			})
			.id();
		let child = app
			.world_mut()
			.spawn((Transform::default(), SkillTransformOf(beam)))
			.id();

		app.update();
		app.update();

		assert_eq!(
			Some(&IsChanged::FALSE),
			app.world().entity(child).get::<IsChanged<Transform>>(),
		);
	}

	#[test]
	fn act_again_if_active_beam_changed() {
		let mut app = setup();
		let beam = app
			.world_mut()
			.spawn(ActiveBeam {
				length: Units::from(42.),
			})
			.id();
		let child = app
			.world_mut()
			.spawn((Transform::default(), SkillTransformOf(beam)))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(beam)
			.get_mut::<ActiveBeam>()
			.as_deref_mut();
		app.update();

		assert_eq!(
			Some(&IsChanged::TRUE),
			app.world().entity(child).get::<IsChanged<Transform>>(),
		);
	}
}
