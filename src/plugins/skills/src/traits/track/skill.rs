use crate::{
	components::{SlotKey, Track},
	skill::{Active, Skill, SkillState, StartBehaviorFn, StopBehaviorFn},
	traits::{Execution, GetSlots},
};
use common::{components::Side, traits::state_duration::StateDuration};
use std::time::Duration;

impl<TData> StateDuration<SkillState> for Track<Skill<TData>> {
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

impl Execution for Track<Skill<Active>> {
	fn get_start(&self) -> Option<StartBehaviorFn> {
		self.value.execution.run_fn
	}

	fn get_stop(&self) -> Option<StopBehaviorFn> {
		self.value.execution.stop_fn
	}
}

impl GetSlots for Track<Skill<Active>> {
	fn slots(&self) -> Vec<SlotKey> {
		match (self.value.data.0, self.value.dual_wield) {
			(SlotKey::Hand(Side::Main), true) => {
				vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Off)]
			}
			(SlotKey::Hand(Side::Off), true) => {
				vec![SlotKey::Hand(Side::Off), SlotKey::Hand(Side::Main)]
			}
			(slot_key, ..) => vec![slot_key],
		}
	}
}

#[cfg(test)]
mod tests_state_duration {
	use super::*;
	use crate::skill::Cast;
	use bevy::utils::default;

	#[test]
	fn get_phasing_times() {
		let track = Track::new(Skill::<()> {
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
		let mut track = Track::new(Skill::<()> {
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
		skill::{SkillExecution, Spawner, Target},
		traits::test_tools::{run_system, stop_system},
	};
	use bevy::{
		app::{App, Update},
		ecs::{entity::Entity, system::EntityCommands},
		math::{Ray3d, Vec3},
		transform::components::{GlobalTransform, Transform},
		utils::default,
	};
	use common::{components::Outdated, resources::ColliderInfo};
	use mockall::mock;

	fn test_target() -> Target {
		Target {
			ray: Ray3d {
				origin: Vec3::Y,
				direction: Vec3::NEG_ONE.try_into().unwrap(),
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
			fn run<'a>(
				_agent: &mut EntityCommands<'a>,
				_agent_transform: &Transform,
				_spawner: &Spawner,
				_target: &Target,
			) {
			}
		}
		impl StopFn for _Tools {
			fn stop<'a>(_agent: &mut EntityCommands<'a>) {}
		}
	}

	#[test]
	fn execute_run() {
		let mut app = App::new();
		let agent = app
			.world
			.spawn(Track::new(Skill::<Active> {
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
			run_system::<Track<Skill<Active>>>(agent, transform, spawner, test_target()),
		);
		app.update();
	}

	#[test]
	fn execute_stop() {
		struct _Tools;

		let mut app = App::new();
		let agent = app
			.world
			.spawn(Track::new(Skill::<Active> {
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

		app.add_systems(Update, stop_system::<Track<Skill<Active>>>(agent));
		app.update();
	}
}

#[cfg(test)]
mod test_get_slot {
	use super::*;
	use bevy::utils::default;
	use common::components::Side;

	#[test]
	fn get_main() {
		let track = Track::new(Skill {
			data: Active(SlotKey::Hand(Side::Off)),
			..default()
		});

		assert_eq!(vec![SlotKey::Hand(Side::Off)], track.slots());
	}

	#[test]
	fn get_off() {
		let track = Track::new(Skill {
			data: Active(SlotKey::Hand(Side::Main)),
			..default()
		});

		assert_eq!(vec![SlotKey::Hand(Side::Main)], track.slots());
	}

	#[test]
	fn get_dual_main() {
		let track = Track::new(Skill {
			data: Active(SlotKey::Hand(Side::Main)),
			dual_wield: true,
			..default()
		});

		assert_eq!(
			vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Off)],
			track.slots()
		);
	}

	#[test]
	fn get_dual_off() {
		let track = Track::new(Skill {
			data: Active(SlotKey::Hand(Side::Off)),
			dual_wield: true,
			..default()
		});

		assert_eq!(
			vec![SlotKey::Hand(Side::Off), SlotKey::Hand(Side::Main)],
			track.slots()
		);
	}

	#[test]
	fn get_skill_spawn() {
		let track = Track::new(Skill {
			data: Active(SlotKey::SkillSpawn),
			..default()
		});

		assert_eq!(vec![SlotKey::SkillSpawn], track.slots());
	}
}
