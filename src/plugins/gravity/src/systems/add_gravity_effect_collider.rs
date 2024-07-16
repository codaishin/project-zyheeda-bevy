use crate::traits::GetGravityEffectCollider;
use bevy::ecs::{
	component::Component,
	entity::Entity,
	query::Added,
	system::{Commands, Query},
};
use common::traits::try_insert_on::TryInsertOn;

pub(crate) fn add_gravity_effect_collider<TGravitySource: Component + GetGravityEffectCollider>(
	mut commands: Commands,
	sources: Query<(Entity, &TGravitySource), Added<TGravitySource>>,
) {
	for (entity, source) in &sources {
		let collider = source.gravity_effect_collider();
		commands.try_insert_on(entity, collider);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::app::{App, Update};
	use bevy_rapier3d::geometry::Collider;
	use common::test_tools::utils::SingleThreadedApp;

	#[derive(Component)]
	struct _Source(Collider);

	impl GetGravityEffectCollider for _Source {
		fn gravity_effect_collider(&self) -> Collider {
			self.0.clone()
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, add_gravity_effect_collider::<_Source>);

		app
	}

	#[test]
	fn add_collider_to_source() {
		let mut app = setup();

		let collider = Collider::ball(0.42);
		let source = app.world_mut().spawn(_Source(collider)).id();

		app.update();

		let source = app.world().entity(source);

		assert_eq!(
			Some(0.42),
			source
				.get::<Collider>()
				.and_then(|c| c.as_ball())
				.map(|b| b.radius())
		);
	}

	#[test]
	fn do_not_add_collider_when_source_not_new() {
		let mut app = setup();

		let collider = Collider::ball(0.42);
		let source = app.world_mut().spawn(_Source(collider)).id();

		app.update();

		app.world_mut().entity_mut(source).remove::<Collider>();

		app.update();

		let source = app.world().entity(source);

		assert_eq!(
			None,
			source
				.get::<Collider>()
				.and_then(|c| c.as_ball())
				.map(|b| b.radius())
		);
	}
}
