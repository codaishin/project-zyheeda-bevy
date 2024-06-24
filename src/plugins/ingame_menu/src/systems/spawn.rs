use crate::traits::{colors::HasBackgroundColor, get_node::GetNode};
use bevy::ecs::{component::Component, system::Commands};

pub fn spawn<TComponent: Default + GetNode + Component + HasBackgroundColor>(
	mut commands: Commands,
) {
	let component = TComponent::default();
	commands.spawn((component.node(), component));
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::tools::assert_node_bundle;
	use bevy::{
		app::{App, Update},
		prelude::default,
		render::color::Color,
		ui::{node_bundles::NodeBundle, BackgroundColor, Style, Val},
	};

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

	impl HasBackgroundColor for _Component {
		const BACKGROUND_COLOR: Option<Color> = Some(Color::rgb(0.1, 0.2, 0.3));
	}

	#[derive(Component, Default)]
	struct _ComponentWithoutBackgroundColor;

	impl GetNode for _ComponentWithoutBackgroundColor {
		fn node(&self) -> NodeBundle {
			NodeBundle {
				style: Style::default(),
				..default()
			}
		}
	}

	impl HasBackgroundColor for _ComponentWithoutBackgroundColor {
		const BACKGROUND_COLOR: Option<Color> = None;
	}

	#[test]
	fn spawn_bundle() {
		let mut app = App::new();

		app.add_systems(Update, spawn::<_Component>);
		app.update();

		let entity_with_component = app
			.world
			.iter_entities()
			.find(|e| e.contains::<_Component>())
			.expect("no _Component spawned");

		assert_node_bundle!(entity_with_component);
		assert_eq!(
			Some(&Style {
				width: Val::Px(42.),
				..default()
			}),
			entity_with_component.get::<Style>()
		)
	}

	#[test]
	fn spawn_bundle_without_background_color() {
		let mut app = App::new();

		app.add_systems(Update, spawn::<_ComponentWithoutBackgroundColor>);
		app.update();

		let entity_with_component = app
			.world
			.iter_entities()
			.find(|e| e.contains::<_ComponentWithoutBackgroundColor>());

		assert_eq!(
			Some(Color::NONE),
			entity_with_component.and_then(|entity| entity
				.get::<BackgroundColor>()
				.map(|background_color| background_color.0))
		)
	}
}
