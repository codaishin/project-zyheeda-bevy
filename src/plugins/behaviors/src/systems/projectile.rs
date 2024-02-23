use crate::{components::SimpleMovement, traits::ProjectileBehavior};
use bevy::{
	ecs::{
		component::Component,
		entity::Entity,
		event::EventReader,
		query::{Added, With},
		system::{Commands, Query},
	},
	hierarchy::DespawnRecursiveExt,
	math::Vec3,
	transform::components::GlobalTransform,
};
use bevy_rapier3d::pipeline::CollisionEvent;
use common::components::{ColliderRoot, Idle};

pub(crate) fn projectile_behavior<TProjectile: ProjectileBehavior + Component>(
	mut commands: Commands,
	mut collision_events: EventReader<CollisionEvent>,
	new_projectiles: Query<(Entity, &TProjectile, &GlobalTransform), Added<TProjectile>>,
	done: Query<Entity, (With<Idle>, With<TProjectile>)>,
	projectiles: Query<Entity, With<TProjectile>>,
	roots: Query<&ColliderRoot>,
) {
	for entity in &done {
		commands.entity(entity).despawn_recursive();
	}

	for (id, projectile, transform) in &new_projectiles {
		let target = get_target(projectile, transform);
		commands.entity(id).insert(SimpleMovement { target });
	}

	for collision in collision_events.read() {
		let CollisionEvent::Started(a, b, ..) = collision else {
			continue;
		};
		if let Ok(projectile) = roots.get(*a).and_then(|r| projectiles.get(r.0)) {
			commands.entity(projectile).despawn_recursive();
		}
		if let Ok(projectile) = roots.get(*b).and_then(|r| projectiles.get(r.0)) {
			commands.entity(projectile).despawn_recursive();
		}
	}
}

fn get_target<TProjectile: ProjectileBehavior>(
	projectile: &TProjectile,
	transform: &GlobalTransform,
) -> Vec3 {
	transform.translation() + projectile.direction() * projectile.range()
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		ecs::component::Component,
		hierarchy::BuildWorldChildren,
		math::Vec3,
	};
	use bevy_rapier3d::{pipeline::CollisionEvent, rapier::geometry::CollisionEventFlags};
	use common::components::ColliderRoot;

	#[derive(Component, Default)]
	struct _Projectile {
		pub direction: Vec3,
		pub range: f32,
	}

	impl ProjectileBehavior for _Projectile {
		fn direction(&self) -> bevy::prelude::Vec3 {
			self.direction
		}
		fn range(&self) -> f32 {
			self.range
		}
	}

	fn setup() -> App {
		let mut app = App::new();
		app.add_systems(Update, projectile_behavior::<_Projectile>);
		app.add_event::<CollisionEvent>();

		app
	}

	#[test]
	fn insert_simple_movement() {
		let mut app = setup();

		let projectile = app
			.world
			.spawn((
				_Projectile {
					direction: Vec3::new(1., 2., 3.),
					range: 42.,
				},
				GlobalTransform::from_translation(Vec3::ZERO),
			))
			.id();

		app.update();

		let projectile = app.world.entity(projectile);

		assert_eq!(
			Some(&SimpleMovement {
				target: Vec3::new(1., 2., 3.) * 42.
			}),
			projectile.get::<SimpleMovement>()
		);
	}

	#[test]
	fn spawn_with_simple_movement_from_offset() {
		let mut app = setup();

		let projectile = app
			.world
			.spawn((
				_Projectile {
					direction: Vec3::new(1., 2., 3.),
					range: 42.,
				},
				GlobalTransform::from_translation(Vec3::new(10., 20., 30.)),
			))
			.id();

		app.update();

		let projectile = app.world.entity(projectile);

		assert_eq!(
			Some(&SimpleMovement {
				target: Vec3::new(10., 20., 30.) + Vec3::new(1., 2., 3.) * 42.
			}),
			projectile.get::<SimpleMovement>()
		);
	}

	#[test]
	fn despawn_when_wait_next_added() {
		#[derive(Component)]
		struct _Child;

		let mut app = setup();

		let projectile = app
			.world
			.spawn((
				_Projectile {
					direction: Vec3::new(1., 2., 3.),
					range: 42.,
				},
				GlobalTransform::from_translation(Vec3::ZERO),
			))
			.with_children(|parent| {
				parent.spawn(_Child);
			})
			.id();

		app.update();

		app.world.entity_mut(projectile).insert(Idle);

		app.update();

		assert_eq!(
			0,
			app.world
				.iter_entities()
				.filter(|entity| entity.contains::<_Child>() || entity.contains::<SimpleMovement>())
				.count()
		);
	}

	#[test]
	fn do_not_despawn_when_projectile_missing() {
		#[derive(Component)]
		struct _Decoy;

		let mut app = setup();

		app.world.spawn((_Decoy, Idle));
		app.update();

		assert_eq!(
			1,
			app.world
				.iter_entities()
				.filter(|entity| entity.contains::<_Decoy>())
				.count()
		);
	}

	#[test]
	fn despawn_on_collision() {
		let mut app = setup();

		let projectile = app
			.world
			.spawn((
				_Projectile::default(),
				GlobalTransform::from_translation(Vec3::ZERO),
			))
			.id();
		let collider = app
			.world
			.spawn(ColliderRoot(projectile))
			.set_parent(projectile)
			.id();

		app.update();

		app.world.send_event(CollisionEvent::Started(
			collider,
			Entity::from_raw(42),
			CollisionEventFlags::SENSOR,
		));

		app.update();

		let projectile = app.world.get_entity(projectile);
		let collider = app.world.get_entity(collider);

		assert_eq!((true, true), (projectile.is_none(), collider.is_none()));
	}

	#[test]
	fn despawn_on_collision_reversed() {
		let mut app = setup();

		let projectile = app
			.world
			.spawn((
				_Projectile::default(),
				GlobalTransform::from_translation(Vec3::ZERO),
			))
			.id();
		let collider = app
			.world
			.spawn(ColliderRoot(projectile))
			.set_parent(projectile)
			.id();

		app.update();

		app.world.send_event(CollisionEvent::Started(
			Entity::from_raw(42),
			collider,
			CollisionEventFlags::SENSOR,
		));

		app.update();

		let projectile = app.world.get_entity(projectile);
		let collider = app.world.get_entity(collider);

		assert_eq!((true, true), (projectile.is_none(), collider.is_none()));
	}
}
