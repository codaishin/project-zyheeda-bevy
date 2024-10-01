use crate::{
	components::{Movement, VelocityBased},
	traits::ProjectileBehavior,
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
use common::{components::Idle, traits::try_insert_on::TryInsertOn};

pub(crate) fn projectile_behavior<TProjectile: ProjectileBehavior + Component>(
	mut commands: Commands,
	projectiles: Query<(Entity, &TProjectile, &GlobalTransform), Added<TProjectile>>,
	done: Query<Entity, (With<Idle>, With<TProjectile>)>,
) {
	for entity in &done {
		commands.entity(entity).despawn_recursive();
	}

	for (id, projectile, transform) in &projectiles {
		let target = get_target(projectile, transform);
		commands.try_insert_on(id, Movement::<VelocityBased>::to(target));
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
		math::{Dir3, Vec3},
	};

	#[derive(Component)]
	struct _Projectile {
		pub direction: Dir3,
		pub range: f32,
	}

	impl ProjectileBehavior for _Projectile {
		fn direction(&self) -> Dir3 {
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
			.world_mut()
			.spawn((
				_Projectile {
					direction: Vec3::new(1., 2., 3.).try_into().unwrap(),
					range: 42.,
				},
				GlobalTransform::from_translation(Vec3::ZERO),
			))
			.id();

		app.update();

		let projectile = app.world().entity(projectile);

		assert_eq!(
			Some(&Movement::<VelocityBased>::to(
				Vec3::new(1., 2., 3.).normalize() * 42.
			)),
			projectile.get::<Movement<VelocityBased>>()
		);
	}

	#[test]
	fn spawn_with_simple_movement_from_offset() {
		let mut app = setup();

		let projectile = app
			.world_mut()
			.spawn((
				_Projectile {
					direction: Vec3::new(1., 2., 3.).try_into().unwrap(),
					range: 42.,
				},
				GlobalTransform::from_translation(Vec3::new(10., 20., 30.)),
			))
			.id();

		app.update();

		let projectile = app.world().entity(projectile);

		assert_eq!(
			Some(&Movement::<VelocityBased>::to(
				Vec3::new(10., 20., 30.) + Vec3::new(1., 2., 3.).normalize() * 42.
			)),
			projectile.get::<Movement<VelocityBased>>()
		);
	}

	#[test]
	fn despawn_when_wait_next_added() {
		#[derive(Component)]
		struct _Child;

		let mut app = setup();

		let projectile = app
			.world_mut()
			.spawn((
				_Projectile {
					direction: Vec3::new(1., 2., 3.).try_into().unwrap(),
					range: 42.,
				},
				GlobalTransform::from_translation(Vec3::ZERO),
			))
			.with_children(|parent| {
				parent.spawn(_Child);
			})
			.id();

		app.update();

		app.world_mut().entity_mut(projectile).insert(Idle);

		app.update();

		assert_eq!(
			0,
			app.world()
				.iter_entities()
				.filter(|entity| entity.contains::<_Child>()
					|| entity.contains::<Movement<VelocityBased>>())
				.count()
		);
	}

	#[test]
	fn do_not_despawn_when_projectile_missing() {
		#[derive(Component)]
		struct _Decoy;

		let mut app = setup();

		app.world_mut().spawn((_Decoy, Idle));
		app.update();

		assert_eq!(
			1,
			app.world()
				.iter_entities()
				.filter(|entity| entity.contains::<_Decoy>())
				.count()
		);
	}
}
