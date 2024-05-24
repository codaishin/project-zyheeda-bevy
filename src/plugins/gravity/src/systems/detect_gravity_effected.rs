use crate::{components::Gravity, traits::GetGravityPull};
use bevy::ecs::{
	component::Component,
	entity::Entity,
	query::{Changed, With},
	system::{Commands, Query},
};
use common::{
	components::effected_by::EffectedBy,
	traits::{
		has_collisions::HasCollisions,
		try_insert_on::TryInsertOn,
		try_remove_from::TryRemoveFrom,
	},
};

type Components<'a, TColliderCollection> = (Entity, &'a TColliderCollection);
type Filter<TColliderCollection> = (With<EffectedBy<Gravity>>, Changed<TColliderCollection>);

pub(crate) fn detect_gravity_effected<
	TColliderCollection: HasCollisions + Component,
	TGravitySource: Component + GetGravityPull,
>(
	mut commands: Commands,
	effected: Query<Components<TColliderCollection>, Filter<TColliderCollection>>,
	gravity_sources: Query<&TGravitySource>,
) {
	let gravity_collision = |entity: Entity| Some((entity, gravity_sources.get(entity).ok()?));

	for (effected, collisions) in &effected {
		match collisions.collisions().find_map(gravity_collision) {
			Some(gravity_contact) => add_gravity_pull(&mut commands, effected, gravity_contact),
			None => remove_gravity_pull(&mut commands, effected),
		}
	}
}

fn add_gravity_pull<TGravitySource: GetGravityPull>(
	commands: &mut Commands,
	effected: Entity,
	(center, source): (Entity, &TGravitySource),
) {
	let pull = source.gravity_pull();
	commands.try_insert_on(effected, Gravity { pull, center })
}

fn remove_gravity_pull(commands: &mut Commands, effected: Entity) {
	commands.try_remove_from::<Gravity>(effected);
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		ecs::entity::Entity,
		utils::default,
	};
	use common::{
		components::effected_by::EffectedBy,
		test_tools::utils::SingleThreadedApp,
		tools::UnitsPerSecond,
		traits::clamp_zero_positive::ClampZeroPositive,
	};

	#[derive(Component, Default)]
	struct _Collisions {
		entities: Vec<Entity>,
		collisions_query_side_effect: Option<fn()>,
	}

	impl HasCollisions for _Collisions {
		fn collisions(&self) -> impl Iterator<Item = Entity> + '_ {
			if let Some(side_effect) = self.collisions_query_side_effect {
				side_effect();
			}
			self.entities.iter().cloned()
		}
	}

	#[derive(Component)]
	struct _GravitySource(UnitsPerSecond);

	impl GetGravityPull for _GravitySource {
		fn gravity_pull(&self) -> UnitsPerSecond {
			self.0
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			detect_gravity_effected::<_Collisions, _GravitySource>,
		);

		app
	}

	#[test]
	fn add_gravity_component_when_colliding() {
		let mut app = setup();
		let source = app
			.world
			.spawn(_GravitySource(UnitsPerSecond::new(42.)))
			.id();
		let agent = app
			.world
			.spawn((
				_Collisions {
					entities: vec![source],
					..default()
				},
				EffectedBy::<Gravity>::default(),
			))
			.id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&Gravity {
				pull: UnitsPerSecond::new(42.),
				center: source,
			}),
			agent.get::<Gravity>(),
		)
	}

	#[test]
	fn remove_gravity_component_when_not_colliding() {
		let mut app = setup();
		let agent = app
			.world
			.spawn((
				_Collisions {
					entities: vec![Entity::from_raw(42)],
					..default()
				},
				EffectedBy::<Gravity>::default(),
				Gravity {
					pull: UnitsPerSecond::new(42.),
					center: Entity::from_raw(11),
				},
			))
			.id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(None, agent.get::<Gravity>(),)
	}

	#[test]
	fn do_not_gravity_component_when_not_effected_by_gravity() {
		struct NotGravity;

		let mut app = setup();
		let source = app
			.world
			.spawn(_GravitySource(UnitsPerSecond::new(42.)))
			.id();
		let agent = app
			.world
			.spawn((
				_Collisions {
					entities: vec![source],
					..default()
				},
				EffectedBy::<NotGravity>::default(),
			))
			.id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(None, agent.get::<Gravity>(),)
	}

	#[test]
	fn query_collisions_only_on_collisions_change() {
		static mut QUERY_COUNT: usize = 0;

		let mut app = setup();
		let source_a = app
			.world
			.spawn(_GravitySource(UnitsPerSecond::new(42.)))
			.id();
		let source_b = app
			.world
			.spawn(_GravitySource(UnitsPerSecond::new(43.)))
			.id();
		let agent = app
			.world
			.spawn((
				_Collisions {
					entities: vec![source_a],
					collisions_query_side_effect: Some(|| unsafe {
						QUERY_COUNT += 1;
					}),
				},
				EffectedBy::<Gravity>::default(),
			))
			.id();

		app.update();
		app.update();

		app.world
			.entity_mut(agent)
			.get_mut::<_Collisions>()
			.unwrap()
			.entities = vec![source_b];

		app.update();

		assert_eq!(2, unsafe { QUERY_COUNT });
	}
}
