use crate::components::lifetime::LifetimeTiedTo;
use bevy::prelude::*;
use common::{
	components::persistent_entity::PersistentEntity,
	traits::accessors::get::{Get, GetProperty, TryApplyOn},
	zyheeda_commands::ZyheedaCommands,
};

impl LifetimeTiedTo {
	pub(crate) fn insert_on<T>(
		trigger: Trigger<OnInsert, T>,
		components: Query<&T>,
		mut commands: ZyheedaCommands,
	) where
		T: Component + GetProperty<PersistentEntity>,
	{
		let entity = trigger.target();
		let Ok(component) = components.get(entity) else {
			return;
		};
		let Some(root) = commands.get(&component.get_property()) else {
			return;
		};

		commands.try_apply_on(&entity, |mut e| {
			e.try_insert(LifetimeTiedTo(root));
		});
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::CommonPlugin;
	use testing::SingleThreadedApp;

	#[derive(Component)]
	struct _Component(PersistentEntity);

	impl GetProperty<PersistentEntity> for _Component {
		fn get_property(&self) -> PersistentEntity {
			self.0
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_plugins(CommonPlugin);
		app.add_observer(LifetimeTiedTo::insert_on::<_Component>);

		app
	}

	#[test]
	fn insert_via_persistent_entity() {
		let mut app = setup();
		let root = PersistentEntity::default();
		let root_id = app.world_mut().spawn(root).id();

		let entity = app.world_mut().spawn(_Component(root));

		assert_eq!(
			Some(&LifetimeTiedTo(root_id)),
			entity.get::<LifetimeTiedTo>(),
		);
	}

	#[test]
	fn act_again_when_reinserted() {
		let mut app = setup();
		let root_a = PersistentEntity::default();
		let root_b = PersistentEntity::default();
		app.world_mut().spawn(root_a);
		let root_b_id = app.world_mut().spawn(root_b).id();
		let mut entity = app.world_mut().spawn(_Component(root_a));

		entity.insert(_Component(root_b));

		assert_eq!(
			Some(&LifetimeTiedTo(root_b_id)),
			entity.get::<LifetimeTiedTo>(),
		);
	}
}
