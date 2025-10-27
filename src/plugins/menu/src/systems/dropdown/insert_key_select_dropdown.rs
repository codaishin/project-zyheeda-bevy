use crate::{
	AppendSkillCommand,
	components::key_select_dropdown_command::{ExcludeKeys, KeySelectDropdownCommand},
	traits::GetComponent,
};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::{
	tools::action_key::slot::SlotKey,
	traits::{
		accessors::get::{EntityContext, TryApplyOn},
		handles_loadout::combos::{Combos, NextConfiguredKeys},
		thread_safe::ThreadSafe,
	},
	zyheeda_commands::ZyheedaCommands,
};

impl KeySelectDropdownCommand<AppendSkillCommand> {
	pub(crate) fn insert_dropdown<TAgent, TLoadout>(
		commands: ZyheedaCommands,
		dropdown_commands: Query<(Entity, &Self)>,
		agents: Query<Entity, With<TAgent>>,
		param: StaticSystemParam<TLoadout>,
	) where
		TAgent: Component,
		TLoadout: for<'c> EntityContext<Combos, TContext<'c>: NextConfiguredKeys<SlotKey>>,
	{
		insert_key_select_dropdown(commands, dropdown_commands, agents, param);
	}
}

fn insert_key_select_dropdown<TAgent, TLoadout, TExtra>(
	mut commands: ZyheedaCommands,
	dropdown_commands: Query<(Entity, &KeySelectDropdownCommand<TExtra>)>,
	agents: Query<Entity, With<TAgent>>,
	param: StaticSystemParam<TLoadout>,
) where
	TAgent: Component,
	TLoadout: for<'c> EntityContext<Combos, TContext<'c>: NextConfiguredKeys<SlotKey>>,
	KeySelectDropdownCommand<TExtra>: ThreadSafe + GetComponent<TInput = ExcludeKeys<SlotKey>>,
{
	for agent in &agents {
		let Some(ctx) = TLoadout::get_entity_context(&param, agent, Combos) else {
			continue;
		};

		for (entity, insert_command) in &dropdown_commands {
			let next_keys = ctx.next_keys(&insert_command.key_path);
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
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::key_select_dropdown_command::ExcludeKeys;
	use common::tools::action_key::slot::PlayerSlot;
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::collections::HashSet;
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Component)]
	struct _Agent;

	#[derive(Component, NestedMocks)]
	struct _Combos {
		mock: Mock_Combos,
	}

	impl Default for _Combos {
		fn default() -> Self {
			let mut mock = Mock_Combos::default();
			mock.expect_next_keys().return_const(HashSet::default());

			Self { mock }
		}
	}

	#[automock]
	impl NextConfiguredKeys<SlotKey> for _Combos {
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

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			insert_key_select_dropdown::<_Agent, Query<Ref<_Combos>>, _Extra>,
		);

		app
	}

	#[test]
	fn spawn_component() {
		let key_path = vec![
			SlotKey::from(PlayerSlot::UPPER_L),
			SlotKey::from(PlayerSlot::LOWER_R),
		];
		let mut app = setup();
		app.world_mut().spawn((
			_Agent,
			_Combos::new().with_mock(|mock| {
				mock.expect_next_keys()
					.times(1)
					.with(eq(key_path.clone()))
					.return_const(HashSet::from([SlotKey::from(PlayerSlot::UPPER_L)]));
			}),
		));
		let entity = app
			.world_mut()
			.spawn(KeySelectDropdownCommand {
				extra: _Extra::Some,
				key_path: key_path.clone(),
			})
			.id();

		app.update();

		assert_eq!(
			Some(&_Component(ExcludeKeys(HashSet::from([SlotKey::from(
				PlayerSlot::UPPER_L
			)])))),
			app.world().entity(entity).get::<_Component>()
		)
	}

	#[test]
	fn remove_insert_command() {
		let mut app = setup();
		app.world_mut().spawn((_Agent, _Combos::default()));
		let entity = app
			.world_mut()
			.spawn(KeySelectDropdownCommand {
				extra: _Extra::Some,
				key_path: vec![],
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
		let mut app = setup();
		app.world_mut().spawn((_Agent, _Combos::default()));
		let entity = app
			.world_mut()
			.spawn(KeySelectDropdownCommand {
				extra: _Extra::None,
				key_path: vec![],
			})
			.id();

		app.update();

		let entity = app.world().get_entity(entity).map(|e| e.id()).ok();
		assert_eq!(None, entity);
	}

	#[test]
	fn despawn_entity_recursively_if_bundle_is_none() {
		let mut app = setup();
		app.world_mut().spawn((_Agent, _Combos::default()));
		let entity = app
			.world_mut()
			.spawn(KeySelectDropdownCommand {
				extra: _Extra::None,
				key_path: vec![],
			})
			.id();
		let child = app.world_mut().spawn(ChildOf(entity)).id();

		app.update();

		let child = app.world().get_entity(child).map(|e| e.id()).ok();
		assert_eq!(None, child);
	}

	#[test]
	fn do_nothing_if_agent_missing() {
		let key_path = vec![
			SlotKey::from(PlayerSlot::UPPER_L),
			SlotKey::from(PlayerSlot::LOWER_R),
		];
		let mut app = setup();
		app.world_mut().spawn(_Combos::new().with_mock(|mock| {
			mock.expect_next_keys().never();
		}));
		app.world_mut().spawn(KeySelectDropdownCommand {
			extra: _Extra::Some,
			key_path: key_path.clone(),
		});

		app.update();
	}
}
