use super::effected_by_gravity::{EffectedByGravity, Pull};
use crate::traits::{ActOn, ActionType};
use bevy::prelude::{Commands, Component, Entity, Query, Transform};
use common::{
	tools::UnitsPerSecond,
	traits::{try_insert_on::TryInsertOn, try_remove_from::TryRemoveFrom},
};
use std::time::Duration;

#[derive(Component, Debug, PartialEq, Clone)]
pub struct Gravity<TExtra = Transform> {
	pub extra: TExtra,
	pub strength: UnitsPerSecond,
}

impl Gravity {
	pub fn pull(strength: UnitsPerSecond) -> Gravity<()> {
		Gravity {
			extra: (),
			strength,
		}
	}

	pub fn set_transform(
		mut commands: Commands,
		agents: Query<(Entity, &Transform, &Gravity<()>)>,
	) {
		for (entity, transform, Gravity { strength, .. }) in &agents {
			commands.try_insert_on(
				entity,
				Gravity {
					extra: *transform,
					strength: *strength,
				},
			);
			commands.try_remove_from::<Gravity<()>>(entity);
		}
	}
}

impl ActOn<EffectedByGravity> for Gravity<Transform> {
	fn act_on(&mut self, target: &mut EffectedByGravity, _: Duration) -> ActionType {
		target.pulls.push(Pull {
			strength: self.strength,
			towards: self.extra.translation,
		});
		ActionType::Always
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{effected_by::EffectedBy, effected_by_gravity::Pull};
	use bevy::{app::App, ecs::system::RunSystemOnce, math::Vec3};
	use common::traits::clamp_zero_positive::ClampZeroPositive;

	#[test]
	fn action_type_always() {
		let mut gravity = Gravity {
			strength: UnitsPerSecond::new(42.),
			extra: Transform::default(),
		};

		let action_type = gravity.act_on(&mut EffectedBy::gravity(), Duration::ZERO);

		assert_eq!(action_type, ActionType::Always);
	}

	#[test]
	fn add_gravity_pull() {
		let mut gravity = Gravity {
			strength: UnitsPerSecond::new(42.),
			extra: Transform::from_xyz(6., 5., 9.),
		};
		let mut effected_by_gravity = EffectedBy::gravity();

		gravity.act_on(&mut effected_by_gravity, Duration::ZERO);

		assert_eq!(
			EffectedByGravity {
				pulls: vec![Pull {
					strength: UnitsPerSecond::new(42.),
					towards: Vec3::new(6., 5., 9.)
				}]
			},
			effected_by_gravity
		);
	}

	#[test]
	fn map_to_gravity_with_transform() {
		let mut app = App::new();
		let transform = Transform::from_xyz(4., 99., -82.);
		let agent = app
			.world_mut()
			.spawn((transform, Gravity::pull(UnitsPerSecond::new(100.))))
			.id();

		app.world_mut().run_system_once(Gravity::set_transform);

		let agent = app.world().entity(agent);
		assert_eq!(
			Some(&Gravity {
				extra: transform,
				strength: UnitsPerSecond::new(100.)
			}),
			agent.get::<Gravity>()
		);
	}

	#[test]
	fn remove_gravity_without_transform() {
		let mut app = App::new();
		let agent = app
			.world_mut()
			.spawn((
				Transform::default(),
				Gravity::pull(UnitsPerSecond::new(100.)),
			))
			.id();

		app.world_mut().run_system_once(Gravity::set_transform);

		let agent = app.world().entity(agent);
		assert_eq!(None, agent.get::<Gravity<()>>());
	}
}
