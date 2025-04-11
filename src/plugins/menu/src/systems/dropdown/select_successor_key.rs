use crate::{
	AppendSkillCommand,
	components::key_select_dropdown_command::{ExcludeKeys, KeySelectDropdownCommand},
	traits::GetComponent,
};
use bevy::prelude::*;
use common::{
	tools::keys::slot::SlotKey,
	traits::{
		handles_combo_menu::NextKeys,
		thread_safe::ThreadSafe,
		try_insert_on::TryInsertOn,
		try_remove_from::TryRemoveFrom,
	},
};

pub(crate) fn select_successor_key<TNextKeys>(
	commands: Commands,
	next_keys: Res<TNextKeys>,
	dropdown_commands: Query<(Entity, &KeySelectDropdownCommand<AppendSkillCommand>)>,
) where
	TNextKeys: Resource + NextKeys,
	KeySelectDropdownCommand<AppendSkillCommand>:
		ThreadSafe + GetComponent<TInput = ExcludeKeys<SlotKey>>,
{
	_select_successor_key(commands, next_keys, dropdown_commands);
}

fn _select_successor_key<TNextKeys, TExtra>(
	mut commands: Commands,
	next_keys: Res<TNextKeys>,
	dropdown_commands: Query<(Entity, &KeySelectDropdownCommand<TExtra>)>,
) where
	TNextKeys: Resource + NextKeys,
	KeySelectDropdownCommand<TExtra>: ThreadSafe + GetComponent<TInput = ExcludeKeys<SlotKey>>,
{
	for (entity, insert_command) in &dropdown_commands {
		let next_keys = next_keys.next_keys(&insert_command.key_path);
		let Some(component) = insert_command.component(ExcludeKeys(next_keys)) else {
			despawn(&mut commands, entity);
			continue;
		};

		commands.try_insert_on(entity, component);
		commands.try_remove_from::<KeySelectDropdownCommand<TExtra>>(entity);
	}
}

fn despawn(commands: &mut Commands, entity: Entity) {
	let Some(entity) = commands.get_entity(entity) else {
		return;
	};

	entity.despawn_recursive();
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::key_select_dropdown_command::ExcludeKeys;
	use common::{
		test_tools::utils::SingleThreadedApp,
		tools::keys::slot::{Side, SlotKey},
		traits::nested_mock::NestedMocks,
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::collections::HashSet;

	#[derive(Resource, NestedMocks)]
	struct _NextKeys {
		mock: Mock_NextKeys,
	}

	impl Default for _NextKeys {
		fn default() -> Self {
			let mut mock = Mock_NextKeys::default();
			mock.expect_next_keys().return_const(HashSet::default());

			Self { mock }
		}
	}

	#[automock]
	impl NextKeys for _NextKeys {
		fn next_keys(&self, combo_keys: &[SlotKey]) -> std::collections::HashSet<SlotKey> {
			self.mock.next_keys(combo_keys)
		}
	}

	#[derive(Debug, PartialEq)]
	enum _Extra {
		Some,
		None,
	}

	impl GetComponent for KeySelectDropdownCommand<_Extra> {
		type TComponent = _Component;
		type TInput = ExcludeKeys<SlotKey>;

		fn component(&self, excluded: Self::TInput) -> Option<Self::TComponent> {
			match self.extra {
				_Extra::None => None,
				_Extra::Some => Some(_Component(excluded)),
			}
		}
	}

	#[derive(Component, Debug, PartialEq, Clone)]
	struct _Component(ExcludeKeys<SlotKey>);

	fn setup(next_keys: _NextKeys) -> App {
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(next_keys);
		app.add_systems(Update, _select_successor_key::<_NextKeys, _Extra>);

		app
	}

	#[test]
	fn spawn_component() {
		let key_path = vec![
			SlotKey::TopHand(Side::Left),
			SlotKey::BottomHand(Side::Right),
		];
		let mut app = setup(_NextKeys::new().with_mock(|mock| {
			mock.expect_next_keys()
				.with(eq(key_path.clone()))
				.return_const(HashSet::from([SlotKey::TopHand(Side::Left)]));
		}));
		let entity = app
			.world_mut()
			.spawn(KeySelectDropdownCommand {
				extra: _Extra::Some,
				key_path: key_path.clone(),
			})
			.id();

		app.update();

		assert_eq!(
			Some(&_Component(ExcludeKeys(HashSet::from([SlotKey::TopHand(
				Side::Left
			)])))),
			app.world().entity(entity).get::<_Component>()
		)
	}

	#[test]
	fn remove_insert_command() {
		let mut app = setup(_NextKeys::default());
		let entity = app
			.world_mut()
			.spawn(KeySelectDropdownCommand {
				extra: _Extra::Some,
				key_path: vec![] as Vec<SlotKey>,
			})
			.id();
		app.update();

		assert_eq!(
			None,
			app.world()
				.entity(entity)
				.get::<KeySelectDropdownCommand<_Extra>>()
		)
	}

	#[test]
	fn despawn_entity_if_bundle_is_none() {
		let mut app = setup(_NextKeys::default());
		let entity = app
			.world_mut()
			.spawn(KeySelectDropdownCommand {
				extra: _Extra::None,
				key_path: vec![] as Vec<SlotKey>,
			})
			.id();

		app.update();

		let entity = app.world().get_entity(entity).map(|e| e.id()).ok();
		assert_eq!(None, entity);
	}

	#[test]
	fn despawn_entity_recursively_if_bundle_is_none() {
		let mut app = setup(_NextKeys::default());
		let entity = app
			.world_mut()
			.spawn(KeySelectDropdownCommand {
				extra: _Extra::None,
				key_path: vec![] as Vec<SlotKey>,
			})
			.id();
		let child = app.world_mut().spawn_empty().set_parent(entity).id();

		app.update();

		let child = app.world().get_entity(child).map(|e| e.id()).ok();
		assert_eq!(None, child);
	}
}
