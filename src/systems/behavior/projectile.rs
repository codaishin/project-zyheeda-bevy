use crate::{
	components::{DequeueNext, SimpleMovement},
	traits::projectile_behavior::ProjectileBehavior,
};
use bevy::{
	ecs::{
		component::Component,
		entity::Entity,
		query::{Added, With},
		system::{Commands, Query},
	},
	hierarchy::DespawnRecursiveExt,
	math::Vec3,
	transform::components::GlobalTransform,
};

pub fn projectile_behavior<TProjectile: ProjectileBehavior + Component>(
	mut commands: Commands,
	projectiles: Query<(Entity, &TProjectile, &GlobalTransform), Added<TProjectile>>,
	done: Query<Entity, (With<DequeueNext>, With<TProjectile>)>,
) {
	for entity in &done {
		commands.entity(entity).despawn_recursive();
	}

	for (id, projectile, transform) in &projectiles {
		let target = get_target(projectile, transform);
		commands.entity(id).insert(SimpleMovement { target });
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
	use crate::components::{DequeueNext, SimpleMovement};
	use bevy::{
		app::{App, Update},
		ecs::component::Component,
		hierarchy::BuildWorldChildren,
		math::Vec3,
	};

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

		app.world.entity_mut(projectile).insert(DequeueNext);

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

		app.world.spawn((_Decoy, DequeueNext));
		app.update();

		assert_eq!(
			1,
			app.world
				.iter_entities()
				.filter(|entity| entity.contains::<_Decoy>())
				.count()
		);
	}
}
