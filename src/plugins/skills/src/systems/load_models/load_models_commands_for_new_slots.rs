use crate::components::{slots::Slots, LoadModel};
use bevy::prelude::{Added, Commands, Entity, Query};
use common::{components::Collection, traits::try_insert_on::TryInsertOn};

pub(crate) fn load_models_commands_for_new_slots(
	mut commands: Commands,
	agents: Query<(Entity, &Slots), Added<Slots>>,
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
		components::{LoadModel, LoadModelsCommand},
		slot_key::SlotKey,
		skills::Skill,
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
			.spawn(Slots::<Skill>::new([(
				SlotKey::BottomHand(Side::Right),
				None,
			)]))
			.id();

		app.update();

		let slots = app.world().entity(slots);

		assert_eq!(
			Some(&LoadModelsCommand::new([LoadModel(SlotKey::BottomHand(
				Side::Right
			))])),
			slots.get::<LoadModelsCommand>()
		);
	}

	#[test]
	fn add_load_models_command_only_when_slots_added() {
		let mut app = setup();

		let slots = app
			.world_mut()
			.spawn(Slots::<Skill>::new([(
				SlotKey::BottomHand(Side::Right),
				None,
			)]))
			.id();

		app.update();

		app.world_mut()
			.entity_mut(slots)
			.get_mut::<Slots<Skill>>()
			.unwrap()
			.0
			.insert(SlotKey::BottomHand(Side::Left), None);

		app.update();

		let slots = app.world().entity(slots);

		assert_eq!(
			Some(&LoadModelsCommand::new([LoadModel(SlotKey::BottomHand(
				Side::Right
			))])),
			slots.get::<LoadModelsCommand>()
		);
	}
}
