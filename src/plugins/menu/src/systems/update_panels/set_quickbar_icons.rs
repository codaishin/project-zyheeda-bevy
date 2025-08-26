use crate::{components::quickbar_panel::QuickbarPanel, tools::PanelState};
use bevy::prelude::*;
use common::{
	tools::action_key::slot::PlayerSlot,
	traits::{
		accessors::get::{RefInto, TryApplyOn},
		handles_loadout_menu::GetItem,
	},
	zyheeda_commands::ZyheedaCommands,
};

pub(crate) fn set_quickbar_icons<TContainer>(
	mut commands: ZyheedaCommands,
	mut panels: Query<(Entity, &mut QuickbarPanel)>,
	containers: Res<TContainer>,
) where
	TContainer: Resource + GetItem<PlayerSlot>,
	TContainer::TItem: for<'a> RefInto<'a, &'a Option<Handle<Image>>>,
{
	for (entity, mut panel) in &mut panels {
		let (state, image) = match containers.get_item(panel.key) {
			Some(item) => (PanelState::Filled, item.ref_into().clone()),
			_ => (PanelState::Empty, None),
		};
		let (state, image) = match image {
			Some(image) => (state, image),
			None => (PanelState::Empty, default()),
		};

		panel.state = state;
		commands.try_apply_on(&entity, |mut e| {
			e.try_insert(ImageNode::new(image));
		});
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{components::quickbar_panel::QuickbarPanel, tools::PanelState};
	use common::tools::action_key::slot::{PlayerSlot, Side};
	use std::collections::HashMap;
	use testing::{SingleThreadedApp, new_handle};

	struct _Item(Option<Handle<Image>>);

	impl<'a> RefInto<'a, &'a Option<Handle<Image>>> for _Item {
		fn ref_into(&self) -> &'_ Option<Handle<Image>> {
			&self.0
		}
	}

	#[derive(Resource)]
	struct _Cache(HashMap<PlayerSlot, _Item>);

	impl GetItem<PlayerSlot> for _Cache {
		type TItem = _Item;

		fn get_item(&self, key: PlayerSlot) -> Option<&Self::TItem> {
			self.0.get(&key)
		}
	}

	impl<const N: usize> From<[(PlayerSlot, _Item); N]> for _Cache {
		fn from(value: [(PlayerSlot, _Item); N]) -> Self {
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
		let key = PlayerSlot::Upper(Side::Right);
		let mut app = setup(_Cache::from([(key, _Item(Some(handle.clone())))]));
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
		let key = PlayerSlot::Upper(Side::Right);
		let mut app = setup(_Cache::from([(key, _Item(None))]));
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
		let key = PlayerSlot::Upper(Side::Right);
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
