use crate::{components::KeySelectDropdownInsertCommand, traits::GetBundle};
use bevy::prelude::{Commands, Component, DespawnRecursiveExt, Entity, Query, With};
use common::traits::{try_insert_on::TryInsertOn, try_remove_from::TryRemoveFrom};

type InsertCommand<TExtra> = KeySelectDropdownInsertCommand<TExtra>;

pub(crate) fn insert_key_select_dropdown<TAgent, TCombos, TExtra>(
	mut commands: Commands,
	agents: Query<&TCombos, With<TAgent>>,
	insert_commands: Query<(Entity, &InsertCommand<TExtra>)>,
) where
	TAgent: Component,
	TCombos: Component,
	TExtra: Sync + Send + 'static,
	for<'a> (&'a InsertCommand<TExtra>, &'a TCombos): GetBundle,
{
	let Ok(combos) = agents.get_single() else {
		return;
	};

	for (entity, insert_command) in &insert_commands {
		let Some(bundle) = (insert_command, combos).bundle() else {
			despawn(&mut commands, entity);
			continue;
		};

		commands.try_insert_on(entity, bundle);
		commands.try_remove_from::<InsertCommand<TExtra>>(entity);
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
	use bevy::{
		app::{App, Update},
		prelude::BuildWorldChildren,
	};
	use common::test_tools::utils::SingleThreadedApp;

	#[derive(Component)]
	struct _Agent;

	#[derive(Component)]
	struct _Combos;

	#[derive(Debug, PartialEq)]
	struct _Extra(Option<_Bundle>);

	#[derive(Component, Debug, PartialEq, Clone)]
	struct _Bundle;

	impl<'a> GetBundle for (&'a InsertCommand<_Extra>, &'a _Combos) {
		type TBundle = _Bundle;

		fn bundle(&self) -> Option<Self::TBundle> {
			let (cmd, ..) = self;
			cmd.extra.0.clone()
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			insert_key_select_dropdown::<_Agent, _Combos, _Extra>,
		);

		app
	}

	#[test]
	fn spawn_bundle() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(InsertCommand {
				extra: _Extra(Some(_Bundle)),
				key_path: vec![],
			})
			.id();
		app.world_mut().spawn((_Agent, _Combos));

		app.update();

		let entity = app.world().entity(entity);

		assert_eq!(Some(&_Bundle), entity.get::<_Bundle>())
	}

	#[test]
	fn do_nothing_if_agent_missing() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(InsertCommand {
				extra: _Extra(Some(_Bundle)),
				key_path: vec![],
			})
			.id();
		app.world_mut().spawn(_Combos);

		app.update();

		let entity = app.world().entity(entity);

		assert_eq!(None, entity.get::<_Bundle>())
	}

	#[test]
	fn remove_insert_command() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(InsertCommand {
				extra: _Extra(Some(_Bundle)),
				key_path: vec![],
			})
			.id();
		app.world_mut().spawn((_Agent, _Combos));

		app.update();

		let entity = app.world().entity(entity);

		assert_eq!(None, entity.get::<InsertCommand<_Extra>>())
	}

	#[test]
	fn despawn_entity_if_bundle_is_none() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(InsertCommand {
				extra: _Extra(None),
				key_path: vec![],
			})
			.id();
		app.world_mut().spawn((_Agent, _Combos));

		app.update();

		let entity = app.world().get_entity(entity).map(|e| e.id());

		assert_eq!(None, entity);
	}

	#[test]
	fn despawn_entity_recursively_if_bundle_is_none() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(InsertCommand {
				extra: _Extra(None),
				key_path: vec![],
			})
			.id();
		let child = app.world_mut().spawn_empty().set_parent(entity).id();
		app.world_mut().spawn((_Agent, _Combos));

		app.update();

		let child = app.world().get_entity(child).map(|e| e.id());

		assert_eq!(None, child);
	}
}
