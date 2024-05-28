use crate::{components::Gravity, traits::GetGravityPull};
use bevy::{
	ecs::{
		bundle::Bundle,
		component::Component,
		entity::Entity,
		query::With,
		system::{Commands, Query},
	},
	math::Vec3,
	transform::components::Transform,
};
use bevy_rapier3d::dynamics::Velocity;
use common::{
	components::{effected_by::EffectedBy, ColliderRoot, Immobilized},
	traits::{has_collisions::HasCollisions, try_insert_on::TryInsertOn},
};
use std::iter::Map;

const SENSITIVITY: f32 = 0.1;

#[derive(Bundle)]
struct ForcedMovement(Velocity, Immobilized);

pub(crate) fn apply_gravity_effect<
	TCollisions: Component + HasCollisions,
	TGravitySource: Component + GetGravityPull,
>(
	mut commands: Commands,
	effected: Query<(Entity, &Transform), With<EffectedBy<Gravity>>>,
	gravities: Query<(&Transform, &TGravitySource, &TCollisions)>,
	collider_roots: Query<&ColliderRoot>,
) {
	let pair_gravity_effected = |(gr_transform, gr_source, collision_entity)| {
		let effected_entity = try_root_entity(collision_entity, &collider_roots);
		let effected = effected.get(effected_entity).ok()?;
		Some((gr_transform, gr_source, effected))
	};
	let gravity_interactions = gravities
		.iter()
		.flat_map(pair_gravity_with_collision)
		.filter_map(pair_gravity_effected);

	for (gr_transform, gr_source, effected) in gravity_interactions {
		let direction = gr_transform.translation - effected.1.translation;
		let pull = *gr_source.gravity_pull();
		let velocity = get_velocity(direction, pull);
		commands.try_insert_on(effected.0, ForcedMovement(velocity, Immobilized));
	}
}

fn pair_gravity_with_collision<'a, TCollisions: HasCollisions, TGravitySource: GetGravityPull>(
	(transform, gravity_source, collisions): (&'a Transform, &'a TGravitySource, &'a TCollisions),
) -> Map<
	impl Iterator<Item = Entity> + 'a,
	impl FnMut(Entity) -> (&'a Transform, &'a TGravitySource, Entity),
> {
	collisions
		.collisions()
		.map(move |collision| (transform, gravity_source, collision))
}

fn try_root_entity(entity: Entity, collider_roots: &Query<&ColliderRoot, ()>) -> Entity {
	let Ok(root) = collider_roots.get(entity) else {
		return entity;
	};

	root.0
}

fn get_velocity(direction: Vec3, pull: f32) -> Velocity {
	if direction.length() < SENSITIVITY * pull {
		Velocity::default()
	} else {
		Velocity::linear(direction.normalize() * pull)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::Gravity;
	use bevy::{
		app::{App, Update},
		ecs::entity::Entity,
		math::Vec3,
		transform::components::Transform,
	};
	use bevy_rapier3d::dynamics::Velocity;
	use common::{
		components::{effected_by::EffectedBy, ColliderRoot, Immobilized},
		test_tools::utils::SingleThreadedApp,
		tools::UnitsPerSecond,
		traits::clamp_zero_positive::ClampZeroPositive,
	};

	#[derive(Component)]
	struct _Pull(UnitsPerSecond);

	impl GetGravityPull for _Pull {
		fn gravity_pull(&self) -> UnitsPerSecond {
			self.0
		}
	}

	#[derive(Component)]
	struct _Collisions(Vec<Entity>);

	impl HasCollisions for _Collisions {
		fn collisions(&self) -> impl Iterator<Item = Entity> + '_ {
			self.0.iter().cloned()
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, apply_gravity_effect::<_Collisions, _Pull>);

		app
	}

	#[test]
	fn pull_colliding_entities_via_velocity() {
		let mut app = setup();
		let agent = app
			.world
			.spawn((
				Transform::from_xyz(10., 0., 0.),
				EffectedBy::<Gravity>::default(),
			))
			.id();
		app.world.spawn((
			Transform::from_xyz(10., 0., 5.),
			_Pull(UnitsPerSecond::new(1.)),
			_Collisions(vec![agent]),
		));

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&Velocity::linear(Vec3::new(0., 0., 1.))),
			agent.get::<Velocity>(),
		)
	}

	#[test]
	fn pull_colliding_entities_scaled_by_pull() {
		let mut app = setup();
		let agent = app
			.world
			.spawn((
				Transform::from_xyz(10., 0., 0.),
				EffectedBy::<Gravity>::default(),
			))
			.id();
		app.world.spawn((
			Transform::from_xyz(10., 0., 5.),
			_Pull(UnitsPerSecond::new(4.)),
			_Collisions(vec![agent]),
		));

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&Velocity::linear(Vec3::new(0., 0., 4.))),
			agent.get::<Velocity>(),
		)
	}

	#[test]
	fn pull_colliding_entities_collider_root() {
		let mut app = setup();
		let agent = app
			.world
			.spawn((
				Transform::from_xyz(10., 0., 0.),
				EffectedBy::<Gravity>::default(),
			))
			.id();
		let root = app.world.spawn(ColliderRoot(agent)).id();
		app.world.spawn((
			Transform::from_xyz(10., 0., 5.),
			_Pull(UnitsPerSecond::new(1.)),
			_Collisions(vec![root]),
		));

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&Velocity::linear(Vec3::new(0., 0., 1.))),
			agent.get::<Velocity>(),
		)
	}

	#[test]
	fn do_not_pull_colliding_entities_when_within_sensitivity_range() {
		let mut app = setup();
		let agent = app
			.world
			.spawn((
				Transform::from_xyz(0., 0., SENSITIVITY - f32::EPSILON),
				EffectedBy::<Gravity>::default(),
			))
			.id();
		app.world.spawn((
			Transform::from_xyz(0., 0., 0.),
			_Pull(UnitsPerSecond::new(1.)),
			_Collisions(vec![agent]),
		));

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(Some(&Velocity::default()), agent.get::<Velocity>(),)
	}

	#[test]
	fn do_not_pull_colliding_entities_when_within_sensitivity_range_scaled_by_pull() {
		let mut app = setup();
		let agent = app
			.world
			.spawn((
				Transform::from_xyz(0., 0., (SENSITIVITY - f32::EPSILON) * 2.),
				EffectedBy::<Gravity>::default(),
			))
			.id();
		app.world.spawn((
			Transform::from_xyz(0., 0., 0.),
			_Pull(UnitsPerSecond::new(2.)),
			_Collisions(vec![agent]),
		));

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(Some(&Velocity::default()), agent.get::<Velocity>(),)
	}

	#[test]
	fn immobilize_colliding_entities() {
		let mut app = setup();
		let agent = app
			.world
			.spawn((
				Transform::from_xyz(10., 0., 0.),
				EffectedBy::<Gravity>::default(),
			))
			.id();
		app.world.spawn((
			Transform::from_xyz(10., 0., 5.),
			_Pull(UnitsPerSecond::new(1.)),
			_Collisions(vec![agent]),
		));

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(Some(&Immobilized), agent.get::<Immobilized>(),)
	}

	#[test]
	fn do_not_effect_when_not_effected_by_gravity() {
		let mut app = setup();
		let agent = app.world.spawn((Transform::from_xyz(10., 0., 0.),)).id();
		app.world.spawn((
			Transform::from_xyz(10., 0., 5.),
			_Pull(UnitsPerSecond::new(1.)),
			_Collisions(vec![agent]),
		));

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			(None, None),
			(agent.get::<Velocity>(), agent.get::<Immobilized>())
		)
	}
}
