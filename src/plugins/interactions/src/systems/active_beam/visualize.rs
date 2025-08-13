use crate::components::{active_beam::ActiveBeam, blockable::Blockable};
use bevy::prelude::*;
use common::{traits::handles_interactions::InteractAble, zyheeda_commands::ZyheedaCommands};

impl ActiveBeam {
	pub(crate) fn visualize(
		mut commands: ZyheedaCommands,
		beams: Query<(Entity, &Blockable), Added<ActiveBeam>>,
	) {
		for (entity, Blockable(beam)) in &beams {
			let InteractAble::Beam { emitter, .. } = beam else {
				continue;
			};

			let container = commands
				.spawn((HALF_FORWARD, Visibility::default(), ChildOf(entity)))
				.id();
			let mut model = commands.spawn(ChildOf(container));

			(emitter.insert_beam_model)(&mut model);
		}
	}
}

const HALF_FORWARD: Transform = Transform::from_translation(Vec3 {
	x: 0.,
	y: 0.,
	z: -0.5,
});

#[cfg(test)]
mod tests {
	use super::*;
	use common::traits::handles_interactions::{BeamEmitter, InteractAble};
	use testing::{SingleThreadedApp, assert_count, get_children};

	#[derive(Component, Debug, PartialEq)]
	struct _Model;

	impl _Model {
		fn insert(entity: &mut EntityCommands) {
			entity.insert(Self);
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, ActiveBeam::visualize);

		app
	}

	#[test]
	fn insert_model_container() {
		let mut app = setup();
		let beam = app
			.world_mut()
			.spawn((
				ActiveBeam,
				Blockable(InteractAble::Beam {
					emitter: BeamEmitter {
						mounted_on: default(),
						range: default(),
						insert_beam_model: _Model::insert,
					},
					blocked_by: default(),
				}),
			))
			.id();

		app.update();

		let [container] = assert_count!(1, get_children!(app, beam, |e| e.id()));
		let [model] = assert_count!(1, get_children!(app, container));
		assert!(model.contains::<_Model>());
	}

	#[test]
	fn insert_model_only_once() {
		let mut app = setup();
		let beam = app
			.world_mut()
			.spawn((
				ActiveBeam,
				Blockable(InteractAble::Beam {
					emitter: BeamEmitter {
						mounted_on: default(),
						range: default(),
						insert_beam_model: _Model::insert,
					},
					blocked_by: default(),
				}),
			))
			.id();

		app.update();
		app.update();

		assert_count!(1, get_children!(app, beam));
	}

	#[test]
	fn insert_container_transform() {
		let mut app = setup();
		let beam = app
			.world_mut()
			.spawn((
				ActiveBeam,
				Blockable(InteractAble::Beam {
					emitter: BeamEmitter {
						mounted_on: default(),
						range: default(),
						insert_beam_model: _Model::insert,
					},
					blocked_by: default(),
				}),
			))
			.id();

		app.update();

		let [container] = assert_count!(1, get_children!(app, beam));
		assert_eq!(Some(&HALF_FORWARD), container.get::<Transform>(),);
	}

	#[test]
	fn insert_container_visibility() {
		let mut app = setup();
		let beam = app
			.world_mut()
			.spawn((
				ActiveBeam,
				Blockable(InteractAble::Beam {
					emitter: BeamEmitter {
						mounted_on: default(),
						range: default(),
						insert_beam_model: _Model::insert,
					},
					blocked_by: default(),
				}),
			))
			.id();

		app.update();

		let [container] = assert_count!(1, get_children!(app, beam));
		assert_eq!(Some(&Visibility::default()), container.get::<Visibility>());
	}
}
