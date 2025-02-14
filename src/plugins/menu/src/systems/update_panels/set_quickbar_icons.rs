use crate::{components::quickbar_panel::QuickbarPanel, tools::PanelState};
use bevy::prelude::*;
use common::{
	tools::slot_key::SlotKey,
	traits::{
		handles_loadout_menus::{GetDescriptor, QuickbarDescriptor},
		try_insert_on::TryInsertOn,
	},
};

pub(crate) fn set_quickbar_icons<TDescriptors>(
	mut commands: Commands,
	mut panels: Query<(Entity, &mut QuickbarPanel)>,
	descriptors: Res<TDescriptors>,
) where
	TDescriptors: Resource + GetDescriptor<SlotKey, TItem = QuickbarDescriptor>,
{
	for (entity, mut panel) in &mut panels {
		let (state, image) = match descriptors.get_descriptor(panel.key) {
			Some(QuickbarDescriptor {
				icon: Some(icon), ..
			}) => (PanelState::Filled, icon.clone()),
			_ => (PanelState::Empty, Handle::default()),
		};

		panel.state = state;
		commands.try_insert_on(entity, ImageNode::new(image));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{components::quickbar_panel::QuickbarPanel, tools::PanelState};
	use common::{
		test_tools::utils::{new_handle, SingleThreadedApp},
		tools::slot_key::{Side, SlotKey},
	};
	use std::collections::HashMap;

	#[derive(Resource)]
	struct _Cache(HashMap<SlotKey, QuickbarDescriptor>);

	impl GetDescriptor<SlotKey> for _Cache {
		type TItem = QuickbarDescriptor;

		fn get_descriptor(&self, key: SlotKey) -> Option<&Self::TItem> {
			self.0.get(&key)
		}
	}

	impl<const N: usize> From<[(SlotKey, QuickbarDescriptor); N]> for _Cache {
		fn from(value: [(SlotKey, QuickbarDescriptor); N]) -> Self {
			Self(HashMap::from(value))
		}
	}

	fn setup(cache: _Cache) -> App {
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(cache);
		app.add_systems(Update, set_quickbar_icons::<_Cache>);

		app
	}

	#[test]
	fn add_icon_image() {
		let handle = new_handle();
		let key = SlotKey::TopHand(Side::Right);
		let mut app = setup(_Cache::from([(
			key,
			QuickbarDescriptor {
				icon: Some(handle.clone()),
				..default()
			},
		)]));
		let panel = app
			.world_mut()
			.spawn(QuickbarPanel {
				state: PanelState::Empty,
				key,
			})
			.id();

		app.update();

		let panel = app.world().entity(panel);
		assert_eq!(
			(Some(&handle), Some(PanelState::Filled)),
			(
				panel.get::<ImageNode>().map(|image| &image.image),
				panel.get::<QuickbarPanel>().map(|panel| panel.state)
			)
		)
	}

	#[test]
	fn set_panel_empty_when_icon_handle_is_none() {
		let key = SlotKey::TopHand(Side::Right);
		let mut app = setup(_Cache::from([(
			key,
			QuickbarDescriptor {
				icon: None,
				..default()
			},
		)]));
		let panel = app
			.world_mut()
			.spawn(QuickbarPanel {
				state: PanelState::Filled,
				key,
			})
			.id();

		app.update();

		let panel = app.world().entity(panel);
		assert_eq!(
			(Some(&Handle::default()), Some(PanelState::Empty)),
			(
				panel.get::<ImageNode>().map(|image| &image.image),
				panel.get::<QuickbarPanel>().map(|panel| panel.state)
			)
		)
	}

	#[test]
	fn set_panel_empty_when_no_descriptor_for_key() {
		let key = SlotKey::TopHand(Side::Right);
		let mut app = setup(_Cache::from([]));
		let panel = app
			.world_mut()
			.spawn(QuickbarPanel {
				state: PanelState::Empty,
				key,
			})
			.id();

		app.update();

		let panel = app.world().entity(panel);
		assert_eq!(
			(Some(&Handle::default()), Some(PanelState::Empty)),
			(
				panel.get::<ImageNode>().map(|image| &image.image),
				panel.get::<QuickbarPanel>().map(|panel| panel.state)
			)
		)
	}
}
