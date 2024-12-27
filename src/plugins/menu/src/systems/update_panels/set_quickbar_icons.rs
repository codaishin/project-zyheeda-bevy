use crate::{components::quickbar_panel::QuickbarPanel, tools::PanelState};
use bevy::prelude::*;
use common::traits::try_insert_on::TryInsertOn;

pub(crate) fn set_quickbar_icons(
	icon_paths: In<Vec<(Entity, Option<Handle<Image>>)>>,
	mut commands: Commands,
	mut panels: Query<&mut QuickbarPanel>,
) {
	for (entity, icon) in icon_paths.0 {
		let Ok(mut panel) = panels.get_mut(entity) else {
			continue;
		};

		let (state, image) = match icon {
			Some(icon) => (PanelState::Filled, icon),
			None => (PanelState::Empty, Handle::default()),
		};

		panel.state = state;
		commands.try_insert_on(entity, ImageNode::new(image));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::tools::PanelState;
	use common::{test_tools::utils::SingleThreadedApp, tools::slot_key::SlotKey};
	use uuid::Uuid;

	#[derive(Resource)]
	struct _Icons(Vec<(Entity, Option<Handle<Image>>)>);

	fn get_icons(data: Res<_Icons>) -> Vec<(Entity, Option<Handle<Image>>)> {
		data.0.clone()
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, get_icons.pipe(set_quickbar_icons));

		app
	}

	fn arbitrary_key() -> SlotKey {
		SlotKey::default()
	}

	#[test]
	fn add_icon_image() {
		let handle = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});

		let mut app = setup();
		let panel = app
			.world_mut()
			.spawn(QuickbarPanel {
				state: PanelState::Empty,
				key: arbitrary_key(),
			})
			.id();
		app.insert_resource(_Icons(vec![(panel, Some(handle.clone()))]));

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
		let mut app = setup();
		let panel = app
			.world_mut()
			.spawn(QuickbarPanel {
				state: PanelState::Filled,
				key: arbitrary_key(),
			})
			.id();
		app.insert_resource(_Icons(vec![(panel, None)]));

		app.update();

		let panel = app.world().entity(panel);

		assert_eq!(
			(Some(&Handle::default()), Some(PanelState::Empty)),
			(
				panel.get::<ImageNode>().map(|image| &image.image),
				panel.get::<QuickbarPanel>().map(|panel| panel.state)
			)
		);
	}
}
