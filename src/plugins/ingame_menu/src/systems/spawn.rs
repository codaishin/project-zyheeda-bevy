use crate::traits::get_node::GetNode;
use bevy::ecs::{component::Component, system::Commands};

pub fn spawn<TComponent: Default + GetNode + Component>(mut commands: Commands) {
	let component = TComponent::default();
	commands.spawn((component.node(), component));
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		prelude::default,
		ui::{node_bundles::NodeBundle, Style, Val},
	};
	use common::assert_bundle;

	#[derive(Component, Default)]
	struct _Component;

	#[derive(Component)]
	struct _Child;

	impl GetNode for _Component {
		fn node(&self) -> NodeBundle {
			NodeBundle {
				style: Style {
					width: Val::Px(42.),
					..default()
				},
				..default()
			}
		}
	}

	#[test]
	fn spawn_bundle() {
		let mut app = App::new();

		app.add_systems(Update, spawn::<_Component>);
		app.update();

		let entity = app
			.world()
			.iter_entities()
			.find(|e| e.contains::<_Component>())
			.expect("no _Component spawned");

		assert_bundle!(
			NodeBundle,
			&app,
			entity,
			With::assert(|style| assert_eq!(
				&Style {
					width: Val::Px(42.),
					..default()
				},
				style
			))
		);
	}
}
