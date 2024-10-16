use crate::{
	components::{Movement, VelocityBased},
	traits::ProjectileBehavior,
};
use bevy::prelude::*;
use common::{components::Idle, traits::try_insert_on::TryInsertOn};

impl<T> ProjectileMovement for T {}

pub(crate) trait ProjectileMovement {
	fn movement(
		mut commands: Commands,
		transforms: Query<&Transform>,
		projectiles: Query<(Entity, &Self, &GlobalTransform), Added<Self>>,
		done: Query<Entity, (With<Idle>, With<Self>)>,
	) where
		Self: ProjectileBehavior + Component + Sized,
	{
		for entity in &done {
			commands.entity(entity).despawn_recursive();
		}

		for (id, projectile, transform) in &projectiles {
			let Ok(caster) = transforms.get(projectile.caster()) else {
				continue;
			};
			let target = projectile.get_target(transform, caster);
			commands.try_insert_on(id, Movement::<VelocityBased>::to(target));
		}
	}

	fn get_target(&self, transform: &GlobalTransform, caster: &Transform) -> Vec3
	where
		Self: ProjectileBehavior,
	{
		transform.translation() + caster.forward() * self.range()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::traits::Caster;
	use common::assert_eq_approx;

	#[derive(Component)]
	struct _Projectile {
		pub caster: Entity,
		pub range: f32,
	}

	impl Caster for _Projectile {
		fn caster(&self) -> Entity {
			self.caster
		}
	}

	impl ProjectileBehavior for _Projectile {
		fn range(&self) -> f32 {
			self.range
		}
	}

	fn setup() -> App {
		let mut app = App::new();
		app.add_systems(Update, _Projectile::movement);

		app
	}

	#[test]
	fn insert_simple_movement() {
		let mut app = setup();
		let caster = app
			.world_mut()
			.spawn(Transform::default().looking_to(Vec3::new(1., 2., 3.), Vec3::Y))
			.id();
		let projectile = app
			.world_mut()
			.spawn((
				_Projectile { caster, range: 42. },
				GlobalTransform::from_translation(Vec3::ZERO),
			))
			.id();

		app.update();

		let projectile = app.world().entity(projectile);

		assert_eq_approx!(
			Some(&Movement::<VelocityBased>::to(
				Vec3::new(1., 2., 3.).normalize() * 42.
			)),
			projectile.get::<Movement<VelocityBased>>(),
			0.00001
		);
	}

	#[test]
	fn spawn_with_simple_movement_from_offset() {
		let mut app = setup();
		let caster = app
			.world_mut()
			.spawn(Transform::default().looking_to(Vec3::new(1., 2., 3.), Vec3::Y))
			.id();
		let projectile = app
			.world_mut()
			.spawn((
				_Projectile { caster, range: 42. },
				GlobalTransform::from_translation(Vec3::new(10., 20., 30.)),
			))
			.id();

		app.update();

		let projectile = app.world().entity(projectile);

		assert_eq_approx!(
			Some(&Movement::<VelocityBased>::to(
				Vec3::new(10., 20., 30.) + Vec3::new(1., 2., 3.).normalize() * 42.
			)),
			projectile.get::<Movement<VelocityBased>>(),
			0.00001
		);
	}

	#[test]
	fn despawn_when_wait_next_added() {
		#[derive(Component)]
		struct _Child;

		let mut app = setup();
		let caster = app
			.world_mut()
			.spawn(Transform::default().looking_to(Vec3::new(1., 2., 3.), Vec3::Y))
			.id();
		let projectile = app
			.world_mut()
			.spawn((
				_Projectile { caster, range: 42. },
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
