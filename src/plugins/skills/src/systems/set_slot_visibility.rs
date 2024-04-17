use crate::components::{SlotKey, SlotVisibility, Slots};
use bevy::{
	ecs::{
		entity::Entity,
		system::{Commands, Query},
	},
	render::view::Visibility,
};
use common::traits::{try_insert_on::TryInsertOn, try_remove_from::TryRemoveFrom};

pub(crate) fn set_slot_visibility(
	mut commands: Commands,
	agents: Query<(Entity, &Slots, &SlotVisibility)>,
) {
	for (id, slots, slot_visibility) in &agents {
		apply_slot_visibility(&mut commands, slots, slot_visibility);
		commands.try_remove_from::<SlotVisibility>(id);
	}
}

fn apply_slot_visibility(commands: &mut Commands, slots: &Slots, slot_visibility: &SlotVisibility) {
	match slot_visibility {
		SlotVisibility::Hidden(keys) => {
			set_visibilities(commands, keys, slots, Visibility::Hidden);
		}
		SlotVisibility::Inherited(keys) => {
			set_visibilities(commands, keys, slots, Visibility::Inherited);
		}
	}
}

fn set_visibilities(
	commands: &mut Commands,
	keys: &[SlotKey],
	slots: &Slots,
	visibility: Visibility,
) {
	for slot in keys.iter().filter_map(|key| slots.0.get(key)) {
		commands.try_insert_on(slot.entity, visibility);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{Slot, SlotKey, SlotVisibility, Slots};
	use bevy::{
		app::{App, Update},
		ecs::entity::Entity,
		render::view::Visibility,
	};
	use common::{components::Side, test_tools::utils::SingleThreadedApp};

	fn setup() -> (App, Entity) {
		let mut app = App::new_single_threaded([Update]);
		let skill_spawn = app.world.spawn_empty().id();
		let main_hand = app.world.spawn_empty().id();
		let off_hand = app.world.spawn_empty().id();
		let agent = app
			.world
			.spawn(Slots(
				[
					(
						SlotKey::SkillSpawn,
						Slot {
							entity: skill_spawn,
							item: None,
						},
					),
					(
						SlotKey::Hand(Side::Main),
						Slot {
							entity: main_hand,
							item: None,
						},
					),
					(
						SlotKey::Hand(Side::Off),
						Slot {
							entity: off_hand,
							item: None,
						},
					),
				]
				.into(),
			))
			.id();
		app.add_systems(Update, set_slot_visibility);

		(app, agent)
	}

	fn get_visibility<'a>(
		slot_key: &SlotKey,
		slots: &Slots,
		app: &'a App,
	) -> Option<&'a Visibility> {
		slots
			.0
			.get(slot_key)
			.and_then(|s| app.world.entity(s.entity).get::<Visibility>())
	}

	#[test]
	fn set_hidden() {
		let (mut app, agent) = setup();
		app.world
			.entity_mut(agent)
			.insert(SlotVisibility::Hidden(vec![
				SlotKey::SkillSpawn,
				SlotKey::Hand(Side::Main),
			]));

		app.update();

		let slots = app.world.entity(agent).get::<Slots>().unwrap();

		assert_eq!(
			(Some(&Visibility::Hidden), Some(&Visibility::Hidden), None),
			(
				get_visibility(&SlotKey::SkillSpawn, slots, &app),
				get_visibility(&SlotKey::Hand(Side::Main), slots, &app),
				get_visibility(&SlotKey::Hand(Side::Off), slots, &app),
			)
		);
	}

	#[test]
	fn set_inherited() {
		let (mut app, agent) = setup();
		app.world
			.entity_mut(agent)
			.insert(SlotVisibility::Inherited(vec![
				SlotKey::SkillSpawn,
				SlotKey::Hand(Side::Off),
			]));

		app.update();

		let slots = app.world.entity(agent).get::<Slots>().unwrap();

		assert_eq!(
			(
				Some(&Visibility::Inherited),
				None,
				Some(&Visibility::Inherited)
			),
			(
				get_visibility(&SlotKey::SkillSpawn, slots, &app),
				get_visibility(&SlotKey::Hand(Side::Main), slots, &app),
				get_visibility(&SlotKey::Hand(Side::Off), slots, &app),
			)
		);
	}

	#[test]
	fn remove_slot_visibility_hidden_component() {
		let (mut app, agent) = setup();
		app.world
			.entity_mut(agent)
			.insert(SlotVisibility::Hidden(vec![]));

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(None, agent.get::<SlotVisibility>());
	}

	#[test]
	fn remove_slot_visibility_inherited_component() {
		let (mut app, agent) = setup();
		app.world
			.entity_mut(agent)
			.insert(SlotVisibility::Inherited(vec![]));

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(None, agent.get::<SlotVisibility>());
	}
}
