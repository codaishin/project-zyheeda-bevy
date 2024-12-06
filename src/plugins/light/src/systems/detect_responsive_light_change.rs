use crate::components::{ChangeLight, ResponsiveLight, ResponsiveLightTrigger};
use bevy::ecs::{
	component::Component,
	entity::Entity,
	query::Changed,
	system::{Commands, Query},
};
use common::traits::{has_collisions::HasCollisions, try_insert_on::TryInsertOn};

pub(crate) fn detect_responsive_light_change<TColliderCollection: HasCollisions + Component>(
	mut commands: Commands,
	responsive_lights: Query<
		(Entity, &ResponsiveLight, &TColliderCollection),
		Changed<TColliderCollection>,
	>,
	triggers: Query<&ResponsiveLightTrigger>,
) {
	for (id, responsive, collisions) in &responsive_lights {
		let change_light = get_change(responsive, collisions, &triggers);
		commands.try_insert_on(id, change_light);
	}
}

fn get_change<TColliderCollection: HasCollisions>(
	responsive: &ResponsiveLight,
	collisions: &TColliderCollection,
	triggers: &Query<&ResponsiveLightTrigger>,
) -> ChangeLight {
	if collisions.collisions().any(|e| triggers.contains(e)) {
		return ChangeLight::Increase(responsive.clone());
	}

	ChangeLight::Decrease(responsive.clone())
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{ChangeLight, ResponsiveLightTrigger};
	use bevy::{
		app::{App, Update},
		asset::{Asset, AssetId, Handle},
		ecs::entity::Entity,
	};
	use common::{
		test_tools::utils::SingleThreadedApp,
		tools::{Intensity, IntensityChangePerSecond, Units},
		traits::clamp_zero_positive::ClampZeroPositive,
	};
	use uuid::Uuid;

	#[derive(Component)]
	struct _Collisions(Vec<Entity>);

	impl HasCollisions for _Collisions {
		fn collisions(&self) -> impl Iterator<Item = Entity> + '_ {
			self.0.iter().cloned()
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, detect_responsive_light_change::<_Collisions>);

		app
	}

	fn new_handle<T: Asset>() -> Handle<T> {
		Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		})
	}

	#[test]
	fn apply_on() {
		let mut app = setup();
		let light_on_material = new_handle();
		let trigger = app.world_mut().spawn(ResponsiveLightTrigger).id();
		let model = app.world_mut().spawn_empty().id();
		let light = app.world_mut().spawn_empty().id();
		let responsive = ResponsiveLight {
			model,
			light,
			range: Units::new(0.),
			light_on_material: light_on_material.clone(),
			light_off_material: new_handle(),
			max: Intensity::new(100.),
			change: IntensityChangePerSecond::new(11.),
		};

		let entity = app
			.world_mut()
			.spawn((responsive.clone(), _Collisions(vec![trigger])))
			.id();

		app.update();

		assert_eq!(
			Some(&ChangeLight::Increase(responsive)),
			app.world().entity(entity).get::<ChangeLight>(),
		)
	}

	#[test]
	fn apply_off() {
		let mut app = setup();
		let light_off_material = new_handle();
		let model = app.world_mut().spawn_empty().id();
		let light = app.world_mut().spawn_empty().id();
		let responsive = ResponsiveLight {
			model,
			light,
			range: Units::new(0.),
			light_on_material: new_handle(),
			light_off_material: light_off_material.clone(),
			max: Intensity::new(100.),
			change: IntensityChangePerSecond::new(11.),
		};

		let entity = app
			.world_mut()
			.spawn((responsive.clone(), _Collisions(vec![])))
			.id();

		app.update();

		assert_eq!(
			Some(&ChangeLight::Decrease(responsive)),
			app.world().entity(entity).get::<ChangeLight>(),
		)
	}
}
