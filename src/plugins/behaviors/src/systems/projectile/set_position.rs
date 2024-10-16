use crate::traits::Spawner;
use bevy::prelude::*;
use common::traits::try_insert_on::TryInsertOn;

impl<T> ProjectileSetPosition for T {}

pub(crate) trait ProjectileSetPosition {
	fn set_position(
		mut commands: Commands,
		transforms: Query<&GlobalTransform>,
		projectiles: Query<(Entity, &Self), Added<Self>>,
	) where
		Self: Spawner + Component + Sized,
	{
		for (entity, projectile) in &projectiles {
			let Ok(spawner) = transforms.get(projectile.spawner()) else {
				continue;
			};
			commands.try_insert_on(entity, *spawner);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::test_tools::utils::SingleThreadedApp;

	#[derive(Component)]
	struct _Projectile {
		spawner: Entity,
	}

	impl Spawner for _Projectile {
		fn spawner(&self) -> Entity {
			self.spawner
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, _Projectile::set_position);

		app
	}

	#[test]
	fn set_spawner_position() {
		let mut app = setup();
		let spawner = app
			.world_mut()
			.spawn(GlobalTransform::from_xyz(1., 2., 3.))
			.id();
		let projectile = app.world_mut().spawn(_Projectile { spawner }).id();

		app.update();

		assert_eq!(
			Some(&GlobalTransform::from_xyz(1., 2., 3.)),
			app.world().entity(projectile).get::<GlobalTransform>(),
		)
	}

	#[test]
	fn set_spawner_position_only_once() {
		let mut app = setup();
		let spawner = app
			.world_mut()
			.spawn(GlobalTransform::from_xyz(1., 2., 3.))
			.id();
		let projectile = app.world_mut().spawn(_Projectile { spawner }).id();

		app.update();
		let mut entity = app.world_mut().entity_mut(projectile);
		let mut transform = entity.get_mut::<GlobalTransform>().unwrap();
		*transform = GlobalTransform::from_xyz(3., 2., 1.);
		app.update();

		assert_eq!(
			Some(&GlobalTransform::from_xyz(3., 2., 1.)),
			app.world().entity(projectile).get::<GlobalTransform>(),
		)
	}
}
