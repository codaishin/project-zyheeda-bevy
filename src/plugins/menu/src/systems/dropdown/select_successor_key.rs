use crate::{
	components::key_select_dropdown_command::{ExcludeKeys, KeySelectDropdownCommand},
	traits::GetComponent,
	AppendSkillCommand,
};
use bevy::prelude::*;
use common::{
	tools::slot_key::SlotKey,
	traits::{
		handles_equipment::FollowupKeys,
		thread_safe::ThreadSafe,
		try_insert_on::TryInsertOn,
		try_remove_from::TryRemoveFrom,
	},
};

pub(crate) fn select_successor_key<TAgent, TCombos>(
	commands: Commands,
	agents: Query<&TCombos, With<TAgent>>,
	dropdown_commands: Query<(Entity, &KeySelectDropdownCommand<AppendSkillCommand>)>,
) where
	TCombos: Component + FollowupKeys<TKey = SlotKey>,
	TAgent: Component,
	KeySelectDropdownCommand<AppendSkillCommand>:
		ThreadSafe + GetComponent<TInput = ExcludeKeys<SlotKey>>,
{
	_select_successor_key(commands, agents, dropdown_commands);
}

fn _select_successor_key<TAgent, TCombos, TExtra>(
	mut commands: Commands,
	agents: Query<&TCombos, With<TAgent>>,
	dropdown_commands: Query<(Entity, &KeySelectDropdownCommand<TExtra>)>,
) where
	TCombos: Component + FollowupKeys<TKey = SlotKey> + Sized,
	TAgent: Component,
	KeySelectDropdownCommand<TExtra>: ThreadSafe + GetComponent<TInput = ExcludeKeys<SlotKey>>,
{
	let Ok(combos) = agents.get_single() else {
		return;
	};

	for (entity, insert_command) in &dropdown_commands {
		let Some(followup_keys) = combos.followup_keys(insert_command.key_path.clone()) else {
			despawn(&mut commands, entity);
			continue;
		};
		let Some(component) = insert_command.component(ExcludeKeys(followup_keys)) else {
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
		tools::slot_key::{Side, SlotKey},
		traits::nested_mock::NestedMocks,
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::collections::VecDeque;

	#[derive(Component)]
	struct _Agent;

	#[derive(Component, NestedMocks)]
	struct _Combos {
		mock: Mock_Combos,
	}

	impl Default for _Combos {
		fn default() -> Self {
			let mut mock = Mock_Combos::default();
			mock.expect_followup_keys::<Vec<_>>()
				.return_const(Some(vec![]));

			Self { mock }
		}
	}

	#[automock]
	impl FollowupKeys for _Combos {
		type TKey = SlotKey;

		fn followup_keys<T>(&self, after: T) -> Option<Vec<Self::TKey>>
		where
			T: Into<VecDeque<SlotKey>> + 'static,
		{
			self.mock.followup_keys(after)
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

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, _select_successor_key::<_Agent, _Combos, _Extra>);

		app
	}

	#[test]
	fn spawn_component() {
		let mut app = setup();
		let key_path = vec![
			SlotKey::TopHand(Side::Left),
			SlotKey::BottomHand(Side::Right),
		];
		let entity = app
			.world_mut()
			.spawn(KeySelectDropdownCommand {
				extra: _Extra::Some,
				key_path: key_path.clone(),
			})
			.id();
		app.world_mut().spawn((
			_Agent,
			_Combos::new().with_mock(|mock| {
				mock.expect_followup_keys::<Vec<_>>()
					.with(eq(key_path.clone()))
					.return_const(Some(vec![SlotKey::TopHand(Side::Left)]));
			}),
		));

		app.update();

		let entity = app.world().entity(entity);

		assert_eq!(
			Some(&_Component(ExcludeKeys(vec![SlotKey::TopHand(Side::Left)]))),
			entity.get::<_Component>()
		)
	}

	#[test]
	fn do_nothing_if_agent_missing() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(KeySelectDropdownCommand {
				extra: _Extra::Some,
				key_path: vec![] as Vec<SlotKey>,
			})
			.id();
		app.world_mut().spawn(_Combos::default());

		app.update();

		let entity = app.world().entity(entity);

		assert_eq!(None, entity.get::<_Component>())
	}

	#[test]
	fn remove_insert_command() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(KeySelectDropdownCommand {
				extra: _Extra::Some,
				key_path: vec![] as Vec<SlotKey>,
			})
			.id();
		app.world_mut().spawn((_Agent, _Combos::default()));

		app.update();

		let entity = app.world().entity(entity);

		assert_eq!(None, entity.get::<KeySelectDropdownCommand<_Extra>>())
	}

	#[test]
	fn despawn_entity_if_followup_keys_is_none() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(KeySelectDropdownCommand {
				extra: _Extra::Some,
				key_path: vec![] as Vec<SlotKey>,
			})
			.id();
		app.world_mut().spawn((
			_Agent,
			_Combos::new().with_mock(|mock| {
				mock.expect_followup_keys::<Vec<_>>().return_const(None);
			}),
		));

		app.update();

		let entity = app.world().get_entity(entity).map(|e| e.id()).ok();

		assert_eq!(None, entity);
	}

	#[test]
	fn despawn_entity_recursively_if_followup_keys_is_none() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(KeySelectDropdownCommand {
				extra: _Extra::Some,
				key_path: vec![] as Vec<SlotKey>,
			})
			.id();
		let child = app.world_mut().spawn_empty().set_parent(entity).id();
		app.world_mut().spawn((
			_Agent,
			_Combos::new().with_mock(|mock| {
				mock.expect_followup_keys::<Vec<_>>().return_const(None);
			}),
		));

		app.update();

		let child = app.world().get_entity(child).map(|e| e.id()).ok();

		assert_eq!(None, child);
	}

	#[test]
	fn despawn_entity_if_bundle_is_none() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(KeySelectDropdownCommand {
				extra: _Extra::None,
				key_path: vec![] as Vec<SlotKey>,
			})
			.id();
		app.world_mut().spawn((_Agent, _Combos::default()));

		app.update();

		let entity = app.world().get_entity(entity).map(|e| e.id()).ok();

		assert_eq!(None, entity);
	}

	#[test]
	fn despawn_entity_recursively_if_bundle_is_none() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(KeySelectDropdownCommand {
				extra: _Extra::None,
				key_path: vec![] as Vec<SlotKey>,
			})
			.id();
		let child = app.world_mut().spawn_empty().set_parent(entity).id();
		app.world_mut().spawn((_Agent, _Combos::default()));

		app.update();

		let child = app.world().get_entity(child).map(|e| e.id()).ok();

		assert_eq!(None, child);
	}
}
