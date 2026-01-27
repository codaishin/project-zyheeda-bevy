use crate::components::lifetime::LifetimeTiedTo;
use bevy::prelude::*;
use common::{
	components::persistent_entity::PersistentEntity,
	traits::accessors::get::{Get, GetProperty, Property, TryApplyOn},
	zyheeda_commands::ZyheedaCommands,
};

impl LifetimeTiedTo {
	pub(crate) fn insert_on<T>(
		trigger: Trigger<OnInsert, T>,
		components: Query<&T>,
		mut commands: ZyheedaCommands,
	) where
		T: Component + GetProperty<LifetimeRoot>,
	{
		let entity = trigger.target();
		let Ok(component) = components.get(entity) else {
			return;
		};
		let Some(root) = Self::get_root(&commands, component.get_property()) else {
			return;
		};

		commands.try_apply_on(&entity, |mut e| {
			e.try_insert(LifetimeTiedTo(root));
		});
	}

	fn get_root(commands: &ZyheedaCommands<'_, '_>, root: LifetimeRoot) -> Option<Entity> {
		match root {
			LifetimeRoot::Persistent(persistent_entity) => commands.get(&persistent_entity),
			LifetimeRoot::Transient(entity) => Some(entity),
		}
	}
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum LifetimeRoot {
	Persistent(PersistentEntity),
	Transient(Entity),
}

impl Property for LifetimeRoot {
	type TValue<'a> = Self;
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::CommonPlugin;
	use testing::SingleThreadedApp;

	#[derive(Component)]
	struct _Component(LifetimeRoot);

	impl GetProperty<LifetimeRoot> for _Component {
		fn get_property(&self) -> LifetimeRoot {
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
	fn insert_via_entity() {
		let mut app = setup();
		let root = app.world_mut().spawn_empty().id();

		let entity = app
			.world_mut()
			.spawn(_Component(LifetimeRoot::Transient(root)));

		assert_eq!(Some(&LifetimeTiedTo(root)), entity.get::<LifetimeTiedTo>(),);
	}

	#[test]
	fn insert_via_persistent_entity() {
		let mut app = setup();
		let root = PersistentEntity::default();
		let root_id = app.world_mut().spawn(root).id();

		let entity = app
			.world_mut()
			.spawn(_Component(LifetimeRoot::Persistent(root)));

		assert_eq!(
			Some(&LifetimeTiedTo(root_id)),
			entity.get::<LifetimeTiedTo>(),
		);
	}

	#[test]
	fn act_again_when_reinserted() {
		let mut app = setup();
		let root_a = app.world_mut().spawn_empty().id();
		let root_b = app.world_mut().spawn_empty().id();
		let mut entity = app
			.world_mut()
			.spawn(_Component(LifetimeRoot::Transient(root_a)));

		entity.insert(_Component(LifetimeRoot::Transient(root_b)));

		assert_eq!(
			Some(&LifetimeTiedTo(root_b)),
			entity.get::<LifetimeTiedTo>(),
		);
	}
}
