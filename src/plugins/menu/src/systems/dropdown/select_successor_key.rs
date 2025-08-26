use crate::{
	AppendSkillCommand,
	components::key_select_dropdown_command::{ExcludeKeys, KeySelectDropdownCommand},
	traits::GetComponent,
};
use bevy::prelude::*;
use common::{
	tools::action_key::slot::PlayerSlot,
	traits::{accessors::get::TryApplyOn, handles_combo_menu::NextConfiguredKeys, thread_safe::ThreadSafe},
	zyheeda_commands::ZyheedaCommands,
};

pub(crate) fn select_successor_key<TNextKeys>(
	commands: ZyheedaCommands,
	next_keys: Res<TNextKeys>,
	dropdown_commands: Query<(Entity, &KeySelectDropdownCommand<AppendSkillCommand>)>,
) where
	TNextKeys: Resource + NextConfiguredKeys<PlayerSlot>,
	KeySelectDropdownCommand<AppendSkillCommand>:
		ThreadSafe + GetComponent<TInput = ExcludeKeys<PlayerSlot>>,
{
	internal_select_successor_key(commands, next_keys, dropdown_commands);
}

fn internal_select_successor_key<TNextKeys, TExtra>(
	mut commands: ZyheedaCommands,
	next_keys: Res<TNextKeys>,
	dropdown_commands: Query<(Entity, &KeySelectDropdownCommand<TExtra>)>,
) where
	TNextKeys: Resource + NextConfiguredKeys<PlayerSlot>,
	KeySelectDropdownCommand<TExtra>: ThreadSafe + GetComponent<TInput = ExcludeKeys<PlayerSlot>>,
{
	for (entity, insert_command) in &dropdown_commands {
		let next_keys = next_keys.next_keys(&insert_command.key_path);
		let Some(component) = insert_command.component(ExcludeKeys(next_keys)) else {
			commands.try_apply_on(&entity, |e| e.try_despawn());
			continue;
		};

		commands.try_apply_on(&entity, |mut e| {
			e.try_insert(component);
			e.try_remove::<KeySelectDropdownCommand<TExtra>>();
		});
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::key_select_dropdown_command::ExcludeKeys;
	use common::tools::action_key::slot::{PlayerSlot, Side};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::collections::HashSet;
	use testing::{NestedMocks, SingleThreadedApp};

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
	impl NextConfiguredKeys<PlayerSlot> for _NextKeys {
		fn next_keys(&self, combo_keys: &[PlayerSlot]) -> std::collections::HashSet<PlayerSlot> {
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
		type TInput = ExcludeKeys<PlayerSlot>;

		fn component(&self, excluded: Self::TInput) -> Option<Self::TComponent> {
			match self.extra {
				_Extra::None => None,
				_Extra::Some => Some(_Component(excluded)),
			}
		}
	}

	#[derive(Component, Debug, PartialEq, Clone)]
	struct _Component(ExcludeKeys<PlayerSlot>);

	fn setup(next_keys: _NextKeys) -> App {
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(next_keys);
		app.add_systems(Update, internal_select_successor_key::<_NextKeys, _Extra>);

		app
	}

	#[test]
	fn spawn_component() {
		let key_path = vec![
			PlayerSlot::Upper(Side::Left),
			PlayerSlot::Lower(Side::Right),
		];
		let mut app = setup(_NextKeys::new().with_mock(|mock| {
			mock.expect_next_keys()
				.with(eq(key_path.clone()))
				.return_const(HashSet::from([PlayerSlot::Upper(Side::Left)]));
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
			Some(&_Component(ExcludeKeys(HashSet::from([
				PlayerSlot::Upper(Side::Left)
			])))),
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
				key_path: vec![] as Vec<PlayerSlot>,
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
				key_path: vec![] as Vec<PlayerSlot>,
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
				key_path: vec![] as Vec<PlayerSlot>,
			})
			.id();
		let child = app.world_mut().spawn(ChildOf(entity)).id();

		app.update();

		let child = app.world().get_entity(child).map(|e| e.id()).ok();
		assert_eq!(None, child);
	}
}
