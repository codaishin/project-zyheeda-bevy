use crate::components::quickbar_panel::QuickbarPanel;
use bevy::prelude::*;
use common::{
	tools::slot_key::SlotKey,
	traits::{
		handles_inventory_menu::GetDescriptor,
		handles_quickbar::{ActiveSlotKey, ComboNextSlotKey},
	},
};

#[allow(clippy::type_complexity)]
pub(crate) fn get_quickbar_icons<TSlots, TActives, TCombos>(
	active_skills: Res<TActives>,
	combo_skills: Res<TCombos>,
	slotted_skills: Res<TSlots>,
	panels: Query<(Entity, &mut QuickbarPanel)>,
) -> Vec<(Entity, Option<Handle<Image>>)>
where
	TSlots: Resource + GetDescriptor<SlotKey>,
	TActives: Resource + GetDescriptor<ActiveSlotKey>,
	TCombos: Resource + GetDescriptor<ComboNextSlotKey>,
{
	let get_icon_path = |(entity, panel): (Entity, &QuickbarPanel)| {
		let icon = combo_skills
			.get_descriptor(ComboNextSlotKey(panel.key))
			.or_else(|| active_skills.get_descriptor(ActiveSlotKey(panel.key)))
			.or_else(|| slotted_skills.get_descriptor(panel.key))
			.and_then(|skill| skill.icon.clone());

		(entity, icon)
	};

	panels.iter().map(get_icon_path).collect()
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{components::quickbar_panel::QuickbarPanel, tools::PanelState};
	use common::{
		test_tools::utils::{new_handle, SingleThreadedApp},
		tools::slot_key::{Side, SlotKey},
		traits::handles_inventory_menu::Descriptor,
	};
	use std::collections::HashMap;

	#[derive(Resource)]
	struct _SlotsIcons(HashMap<SlotKey, Descriptor>);

	impl<const N: usize> From<[(SlotKey, Descriptor); N]> for _SlotsIcons {
		fn from(value: [(SlotKey, Descriptor); N]) -> Self {
			Self(HashMap::from(value))
		}
	}

	impl GetDescriptor<SlotKey> for _SlotsIcons {
		fn get_descriptor(&self, key: SlotKey) -> Option<&Descriptor> {
			self.0.get(&key)
		}
	}

	#[derive(Resource)]
	struct _ActiveSlotsIcons(HashMap<ActiveSlotKey, Descriptor>);

	impl GetDescriptor<ActiveSlotKey> for _ActiveSlotsIcons {
		fn get_descriptor(&self, key: ActiveSlotKey) -> Option<&Descriptor> {
			self.0.get(&key)
		}
	}

	impl<const N: usize> From<[(ActiveSlotKey, Descriptor); N]> for _ActiveSlotsIcons {
		fn from(value: [(ActiveSlotKey, Descriptor); N]) -> Self {
			Self(HashMap::from(value))
		}
	}

	#[derive(Resource)]
	struct _ComboSlotsIcons(HashMap<ComboNextSlotKey, Descriptor>);

	impl GetDescriptor<ComboNextSlotKey> for _ComboSlotsIcons {
		fn get_descriptor(&self, key: ComboNextSlotKey) -> Option<&Descriptor> {
			self.0.get(&key)
		}
	}

	impl<const N: usize> From<[(ComboNextSlotKey, Descriptor); N]> for _ComboSlotsIcons {
		fn from(value: [(ComboNextSlotKey, Descriptor); N]) -> Self {
			Self(HashMap::from(value))
		}
	}

	#[derive(Resource, Default)]
	struct _Result(Vec<(Entity, Option<Handle<Image>>)>);

	fn store_result(result: In<Vec<(Entity, Option<Handle<Image>>)>>, mut commands: Commands) {
		commands.insert_resource(_Result(result.0));
	}

	fn setup(slots: _SlotsIcons, active: _ActiveSlotsIcons, combos: _ComboSlotsIcons) -> App {
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(slots);
		app.insert_resource(active);
		app.insert_resource(combos);
		app.init_resource::<_Result>();
		app.add_systems(
			Update,
			get_quickbar_icons::<_SlotsIcons, _ActiveSlotsIcons, _ComboSlotsIcons>
				.pipe(store_result),
		);

		app
	}

	#[test]
	fn return_combo_skill_icon() {
		let icon = Some(new_handle());
		let key = SlotKey::BottomHand(Side::Right);
		let mut app = setup(
			_SlotsIcons::from([]),
			_ActiveSlotsIcons::from([]),
			_ComboSlotsIcons::from([(
				ComboNextSlotKey(key),
				Descriptor {
					icon: icon.clone(),
					..default()
				},
			)]),
		);
		let panel = app
			.world_mut()
			.spawn(QuickbarPanel {
				key,
				state: PanelState::Empty,
			})
			.id();

		app.update();

		let result = app.world().resource::<_Result>();
		assert_eq!(vec![(panel, icon)], result.0);
	}

	#[test]
	fn return_active_skill_when_no_combo() {
		let icon = Some(new_handle());
		let key = SlotKey::BottomHand(Side::Right);
		let mut app = setup(
			_SlotsIcons::from([]),
			_ActiveSlotsIcons::from([(
				ActiveSlotKey(key),
				Descriptor {
					icon: icon.clone(),
					..default()
				},
			)]),
			_ComboSlotsIcons::from([]),
		);
		let panel = app
			.world_mut()
			.spawn(QuickbarPanel {
				key,
				state: PanelState::Empty,
			})
			.id();

		app.update();

		let result = app.world().resource::<_Result>();
		assert_eq!(vec![(panel, icon)], result.0);
	}

	#[test]
	fn return_slot_skill_icon_no_combo_or_active_icon() {
		let icon = Some(new_handle());
		let key = SlotKey::BottomHand(Side::Right);
		let mut app = setup(
			_SlotsIcons::from([(
				key,
				Descriptor {
					icon: icon.clone(),
					..default()
				},
			)]),
			_ActiveSlotsIcons::from([]),
			_ComboSlotsIcons::from([]),
		);
		let panel = app
			.world_mut()
			.spawn(QuickbarPanel {
				key,
				state: PanelState::Empty,
			})
			.id();

		app.update();

		let result = app.world().resource::<_Result>();
		assert_eq!(vec![(panel, icon)], result.0);
	}
}
