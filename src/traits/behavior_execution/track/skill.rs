use crate::{
	behaviors::meta::Spawner,
	components::Track,
	skill::{Active, Skill},
	traits::behavior_execution::BehaviorExecution,
};
use bevy::{ecs::system::EntityCommands, transform::components::Transform};

impl<TAnimationKey> BehaviorExecution for Track<Skill<TAnimationKey, Active>> {
	fn run(&self, agent: &mut EntityCommands, agent_transform: &Transform, spawner: &Spawner) {
		let Some(run) = self.value.behavior.run_fn else {
			return;
		};
		run(
			agent,
			agent_transform,
			spawner,
			&self.value.data.select_info,
		);
	}

	fn stop(&self, agent: &mut EntityCommands) {
		let Some(stop) = self.value.behavior.stop_fn else {
			return;
		};
		stop(agent);
	}

	fn apply_transform(&self, transform: &mut Transform, spawner: &Spawner) {
		let Some(apply_transform) = self.value.behavior.transform_fn else {
			return;
		};
		apply_transform(transform, spawner, &self.value.data.select_info);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		behaviors::meta::{BehaviorMeta, Outdated, Target},
		components::{PlayerSkills, SideUnset},
		resources::MouseHover,
		traits::behavior_execution::test_tools::{run_system, stop_system},
	};
	use bevy::{
		app::{App, Update},
		ecs::entity::Entity,
		math::{Ray, Vec3},
		transform::components::{GlobalTransform, Transform},
		utils::default,
	};
	use mockall::{mock, predicate::eq};

	fn test_target() -> Target {
		Target {
			ray: Ray {
				origin: Vec3::Y,
				direction: Vec3::NEG_ONE,
			},
			hover: MouseHover {
				collider: Some(Outdated {
					entity: Entity::from_raw(42),
					transform: GlobalTransform::from_xyz(0., 4., 2.),
				}),
				root: Some(Outdated {
					entity: Entity::from_raw(420),
					transform: GlobalTransform::from_xyz(4., 2., 0.),
				}),
			},
		}
	}

	struct _Tools;

	trait StartFn {
		fn run(
			_agent: &mut EntityCommands,
			_agent_transform: &Transform,
			_spawner: &Spawner,
			_target: &Target,
		);
	}

	trait StopFn {
		fn stop(_agent: &mut EntityCommands);
	}

	trait TransformFn {
		fn transform(_agent: &mut Transform, _spawner: &Spawner, _ray: &Target);
	}

	mock! {
		_Tools{}
		impl StartFn for _Tools {
			fn run<'a, 'b, 'c>(
				_agent: &mut EntityCommands<'a, 'b, 'c>,
				_agent_transform: &Transform,
				_spawner: &Spawner,
				_target: &Target,
			) {
			}
		}
		impl StopFn for _Tools {
			fn stop<'a, 'b, 'c>(_agent: &mut EntityCommands<'a, 'b, 'c>) {}
		}
		impl TransformFn for _Tools {
			fn transform(_agent: &mut Transform, _spawner: &Spawner, _target: &Target) {}
		}
	}

	#[test]
	fn execute_run() {
		let mut app = App::new();
		let agent = app
			.world
			.spawn(Track::new(Skill {
				data: Active {
					select_info: test_target(),
					..default()
				},
				behavior: BehaviorMeta {
					run_fn: Some(Mock_Tools::run),
					..default()
				},
				..default()
			}))
			.id();
		let transform = Transform::from_xyz(1., 2., 3.);
		let spawner = Spawner(GlobalTransform::from_xyz(1., 2., 3.));
		let ctx = Mock_Tools::run_context();
		ctx.expect()
			.times(1)
			.withf(move |a, t, s, i| {
				a.id() == agent && *t == transform && *s == spawner && *i == test_target()
			})
			.return_const(());

		app.add_systems(
			Update,
			run_system::<Track<Skill<PlayerSkills<SideUnset>, Active>>>(agent, transform, spawner),
		);
		app.update();
	}

	#[test]
	fn execute_stop() {
		struct _Tools;

		let mut app = App::new();
		let agent = app
			.world
			.spawn(Track::new(Skill {
				data: Active { ..default() },
				behavior: BehaviorMeta {
					stop_fn: Some(Mock_Tools::stop),
					..default()
				},
				..default()
			}))
			.id();
		let ctx = Mock_Tools::stop_context();
		ctx.expect()
			.times(1)
			.withf(move |a| a.id() == agent)
			.return_const(());

		app.add_systems(
			Update,
			stop_system::<Track<Skill<PlayerSkills<SideUnset>, Active>>>(agent),
		);
		app.update();
	}

	#[test]
	fn execute_transform_fn() {
		let mut transform = Transform::from_xyz(11., 12., 13.);
		let spawner = Spawner(GlobalTransform::from_xyz(22., 33., 44.));
		let track = Track::new(Skill {
			data: Active {
				select_info: test_target(),
				..default()
			},
			behavior: BehaviorMeta {
				transform_fn: Some(Mock_Tools::transform),
				..default()
			},
			..default()
		});

		let ctx = Mock_Tools::transform_context();
		ctx.expect()
			.times(1)
			.with(eq(transform), eq(spawner), eq(test_target()))
			.return_const(());

		track.apply_transform(&mut transform, &spawner);
	}
}
