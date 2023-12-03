use super::BehaviorExecution;
use crate::{
	behaviors::meta::Spawner,
	components::{Active, Skill},
};
use bevy::{ecs::system::EntityCommands, transform::components::Transform};

impl BehaviorExecution for Skill<Active> {
	fn run(&self, agent: &mut EntityCommands, spawner: &Spawner) {
		let Some(run) = self.behavior.run_fn else {
			return;
		};
		run(agent, spawner, &self.data.ray);
	}

	fn stop(&self, agent: &mut EntityCommands) {
		let Some(stop) = self.behavior.stop_fn else {
			return;
		};
		stop(agent);
	}

	fn apply_transform(&self, transform: &mut Transform, spawner: &Spawner) {
		let Some(apply_transform) = self.behavior.transform_fn else {
			return;
		};
		apply_transform(transform, spawner, &self.data.ray);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		behaviors::meta::BehaviorMeta,
		traits::behavior_execution::test_tools::{run_system, stop_system},
	};
	use bevy::{
		app::{App, Update},
		math::{Ray, Vec3},
		transform::components::{GlobalTransform, Transform},
		utils::default,
	};
	use mockall::{automock, predicate::eq};

	const TEST_RAY: Ray = Ray {
		origin: Vec3::Y,
		direction: Vec3::NEG_ONE,
	};

	#[test]
	fn execute_run() {
		struct _Tools;

		trait StartFn {
			fn run(_agent: &mut EntityCommands, _spawner: &Spawner, _ray: &Ray);
		}

		#[automock]
		impl StartFn for _Tools {
			#[allow(clippy::needless_lifetimes)]
			fn run<'a, 'b, 'c>(
				_agent: &mut EntityCommands<'a, 'b, 'c>,
				_spawner: &Spawner,
				_ray: &Ray,
			) {
			}
		}

		let mut app = App::new();
		let agent = app
			.world
			.spawn(Skill {
				data: Active {
					ray: TEST_RAY,
					..default()
				},
				behavior: BehaviorMeta {
					run_fn: Some(Mock_Tools::run),
					..default()
				},
				..default()
			})
			.id();
		let spawner = Spawner(GlobalTransform::from_xyz(1., 2., 3.));
		let ctx = Mock_Tools::run_context();
		ctx.expect()
			.times(1)
			.withf(move |a, s, r| a.id() == agent && *s == spawner && *r == TEST_RAY)
			.return_const(());

		app.add_systems(Update, run_system::<Skill<Active>>(agent, spawner));
		app.update();
	}

	#[test]
	fn execute_stop() {
		struct _Tools;

		trait StopFn {
			fn stop(_agent: &mut EntityCommands);
		}

		#[automock]
		impl StopFn for _Tools {
			#[allow(clippy::needless_lifetimes)]
			fn stop<'a, 'b, 'c>(_agent: &mut EntityCommands<'a, 'b, 'c>) {}
		}

		let mut app = App::new();
		let agent = app
			.world
			.spawn(Skill {
				data: Active { ..default() },
				behavior: BehaviorMeta {
					stop_fn: Some(Mock_Tools::stop),
					..default()
				},
				..default()
			})
			.id();
		let ctx = Mock_Tools::stop_context();
		ctx.expect()
			.times(1)
			.withf(move |a| a.id() == agent)
			.return_const(());

		app.add_systems(Update, stop_system::<Skill<Active>>(agent));
		app.update();
	}

	#[test]
	fn execute_transform_fn() {
		struct _Tools;

		trait StopFn {
			fn transform(_agent: &mut Transform, _spawner: &Spawner, _ray: &Ray);
		}

		#[automock]
		impl StopFn for _Tools {
			fn transform(_agent: &mut Transform, _spawner: &Spawner, _ray: &Ray) {}
		}

		let mut transform = Transform::from_xyz(11., 12., 13.);
		let spawner = Spawner(GlobalTransform::from_xyz(22., 33., 44.));
		let skill = Skill {
			data: Active {
				ray: TEST_RAY,
				..default()
			},
			behavior: BehaviorMeta {
				transform_fn: Some(Mock_Tools::transform),
				..default()
			},
			..default()
		};

		let ctx = Mock_Tools::transform_context();
		ctx.expect()
			.times(1)
			.with(eq(transform), eq(spawner), eq(TEST_RAY))
			.return_const(());

		skill.apply_transform(&mut transform, &spawner);
	}
}
