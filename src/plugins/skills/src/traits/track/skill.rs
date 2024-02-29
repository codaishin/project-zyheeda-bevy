use crate::{
	components::Track,
	skill::{Active, Skill, SkillState, Spawner},
	traits::Execution,
};
use bevy::{ecs::system::EntityCommands, transform::components::Transform};
use common::traits::state_duration::StateDuration;
use std::time::Duration;

impl<TAnimationKey, TData> StateDuration<SkillState> for Track<Skill<TAnimationKey, TData>> {
	fn elapsed_mut(&mut self) -> &mut Duration {
		&mut self.elapsed
	}

	fn get_state_duration(&self, key: SkillState) -> Duration {
		match key {
			SkillState::Aim => self.value.cast.aim,
			SkillState::PreCast => self.value.cast.pre,
			SkillState::Active => self.value.cast.active,
			SkillState::AfterCast => self.value.cast.after,
		}
	}
}

impl<TAnimationKey> Execution for Track<Skill<TAnimationKey, Active>> {
	fn run(&self, agent: &mut EntityCommands, agent_transform: &Transform, spawner: &Spawner) {
		let Some(run) = self.value.execution.run_fn else {
			return;
		};
		run(agent, agent_transform, spawner, &self.value.data.target);
	}

	fn stop(&self, agent: &mut EntityCommands) {
		let Some(stop) = self.value.execution.stop_fn else {
			return;
		};
		stop(agent);
	}
}

#[cfg(test)]
mod tests_state_duration {
	use super::*;
	use crate::skill::Cast;
	use bevy::utils::default;

	#[test]
	fn get_phasing_times() {
		let track = Track::new(Skill::<(), ()> {
			data: (),
			cast: Cast {
				aim: Duration::from_millis(42),
				pre: Duration::from_millis(1),
				active: Duration::from_millis(2),
				after: Duration::from_millis(3),
			},
			..default()
		});

		assert_eq!(
			[
				(Duration::from_millis(42), SkillState::Aim),
				(Duration::from_millis(1), SkillState::PreCast),
				(Duration::from_millis(2), SkillState::Active),
				(Duration::from_millis(3), SkillState::AfterCast),
			],
			[
				SkillState::Aim,
				SkillState::PreCast,
				SkillState::Active,
				SkillState::AfterCast
			]
			.map(|state| (track.get_state_duration(state), state))
		)
	}

	#[test]
	fn get_duration() {
		let mut track = Track::new(Skill::<(), ()> {
			data: (),
			..default()
		});

		*track.elapsed_mut() = Duration::from_secs(42);

		assert_eq!(Duration::from_secs(42), track.elapsed);
	}
}

#[cfg(test)]
mod tests_execution {
	use super::*;
	use crate::{
		components::SideUnset,
		skill::{PlayerSkills, SkillExecution, Target},
		traits::test_tools::{run_system, stop_system},
	};
	use bevy::{
		app::{App, Update},
		ecs::entity::Entity,
		math::{Ray, Vec3},
		transform::components::{GlobalTransform, Transform},
		utils::default,
	};
	use common::{components::Outdated, resources::ColliderInfo};
	use mockall::mock;

	fn test_target() -> Target {
		Target {
			ray: Ray {
				origin: Vec3::Y,
				direction: Vec3::NEG_ONE,
			},
			collision_info: Some(ColliderInfo {
				collider: Outdated {
					entity: Entity::from_raw(42),
					component: GlobalTransform::from_xyz(0., 4., 2.),
				},
				root: Some(Outdated {
					entity: Entity::from_raw(420),
					component: GlobalTransform::from_xyz(4., 2., 0.),
				}),
			}),
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
	}

	#[test]
	fn execute_run() {
		let mut app = App::new();
		let agent = app
			.world
			.spawn(Track::new(Skill::<PlayerSkills<SideUnset>, Active> {
				data: Active {
					target: test_target(),
					..default()
				},
				execution: SkillExecution {
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
			.spawn(Track::new(Skill::<PlayerSkills<SideUnset>, Active> {
				data: Active { ..default() },
				execution: SkillExecution {
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
}
