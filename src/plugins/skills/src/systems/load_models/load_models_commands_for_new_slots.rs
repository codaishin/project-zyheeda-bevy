use crate::{
	components::{slots::Slots, LoadModel},
	skills::Skill,
};
use bevy::{
	asset::Handle,
	prelude::{Added, Commands, Entity, Query},
};
use common::{components::Collection, traits::try_insert_on::TryInsertOn};

type SkillSlots = Slots<Handle<Skill>>;

pub(crate) fn load_models_commands_for_new_slots(
	mut commands: Commands,
	agents: Query<(Entity, &SkillSlots), Added<SkillSlots>>,
) {
	for (agent, slot) in &agents {
		let command = Collection(slot.0.keys().cloned().map(LoadModel).collect());
		commands.try_insert_on(agent, command);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{LoadModel, LoadModelsCommand, Mounts, Slot},
		items::slot_key::SlotKey,
	};
	use bevy::app::{App, Update};
	use common::{components::Side, test_tools::utils::SingleThreadedApp};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, load_models_commands_for_new_slots);

		app
	}

	#[test]
	fn add_load_models_command_when() {
		let mut app = setup();

		let slots = app
			.world_mut()
			.spawn(Slots::<Handle<Skill>>::new([(
				SlotKey::Hand(Side::Main),
				Slot {
					mounts: Mounts {
						hand: Entity::from_raw(42),
						forearm: Entity::from_raw(24),
					},
					item: None,
				},
			)]))
			.id();

		app.update();

		let slots = app.world().entity(slots);

		assert_eq!(
			Some(&LoadModelsCommand::new([LoadModel(SlotKey::Hand(
				Side::Main
			))])),
			slots.get::<LoadModelsCommand>()
		);
	}

	#[test]
	fn add_load_models_command_only_when_slots_added() {
		let mut app = setup();

		let slots = app
			.world_mut()
			.spawn(Slots::<Handle<Skill>>::new([(
				SlotKey::Hand(Side::Main),
				Slot {
					mounts: Mounts {
						hand: Entity::from_raw(42),
						forearm: Entity::from_raw(24),
					},
					item: None,
				},
			)]))
			.id();

		app.update();

		app.world_mut()
			.entity_mut(slots)
			.get_mut::<Slots<Handle<Skill>>>()
			.unwrap()
			.0
			.insert(
				SlotKey::Hand(Side::Off),
				Slot {
					mounts: Mounts {
						hand: Entity::from_raw(42),
						forearm: Entity::from_raw(24),
					},
					item: None,
				},
			);

		app.update();

		let slots = app.world().entity(slots);

		assert_eq!(
			Some(&LoadModelsCommand::new([LoadModel(SlotKey::Hand(
				Side::Main
			))])),
			slots.get::<LoadModelsCommand>()
		);
	}
}
