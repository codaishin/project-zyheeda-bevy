use crate::plugins::ingame_menu::traits::{colors::GetBaseColors, spawn_able::SpawnAble};
use bevy::{
	ecs::{component::Component, system::Commands},
	hierarchy::BuildChildren,
};

pub fn spawn<TComponent: SpawnAble + Component, TGetBaseColors: GetBaseColors>(
	mut commands: Commands,
) {
	let colors = TGetBaseColors::get_base_colors();

	commands
		.spawn(TComponent::bundle(colors))
		.with_children(|parent| TComponent::children(colors, parent));
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::plugins::ingame_menu::traits::colors::BaseColors;
	use bevy::{
		app::{App, Update},
		hierarchy::Parent,
		prelude::default,
		render::color::Color,
		ui::{node_bundles::NodeBundle, Style, Val},
	};

	#[derive(Component)]
	struct _Component(BaseColors);

	#[derive(Component)]
	struct _Child(BaseColors);

	impl SpawnAble for _Component {
		fn bundle(colors: BaseColors) -> (bevy::prelude::NodeBundle, Self) {
			(
				NodeBundle {
					style: Style {
						width: Val::Px(42.),
						..default()
					},
					..default()
				},
				Self(colors),
			)
		}

		fn children(colors: BaseColors, parent: &mut bevy::prelude::ChildBuilder) {
			parent.spawn(_Child(colors));
		}
	}

	struct _Colors;

	impl GetBaseColors for _Colors {
		fn get_base_colors() -> BaseColors {
			BaseColors {
				background: Color::rgb(0.1, 0.2, 0.3),
				text: Color::rgb(0.3, 0.2, 0.1),
			}
		}
	}

	#[test]
	fn spawn_bundle() {
		let mut app = App::new();

		app.add_systems(Update, spawn::<_Component, _Colors>);
		app.update();

		let entity_with_component = app
			.world
			.iter_entities()
			.find(|e| e.contains::<_Component>());

		assert_eq!(
			Some(&Style {
				width: Val::Px(42.),
				..default()
			}),
			entity_with_component.and_then(|e| e.get::<Style>())
		)
	}

	#[test]
	fn bundle_colors() {
		let mut app = App::new();

		app.add_systems(Update, spawn::<_Component, _Colors>);
		app.update();

		let component = app
			.world
			.iter_entities()
			.find_map(|e| e.get::<_Component>())
			.unwrap();

		assert_eq!(_Colors::get_base_colors(), component.0);
	}

	#[test]
	fn spawn_children() {
		let mut app = App::new();

		app.add_systems(Update, spawn::<_Component, _Colors>);
		app.update();

		let entity_with_component = app
			.world
			.iter_entities()
			.find(|e| e.contains::<_Component>())
			.unwrap()
			.id();
		let child_parent = app
			.world
			.iter_entities()
			.find(|e| e.contains::<_Child>())
			.and_then(|e| e.get::<Parent>());

		assert_eq!(Some(entity_with_component), child_parent.map(|p| p.get()))
	}

	#[test]
	fn children_colors() {
		let mut app = App::new();

		app.add_systems(Update, spawn::<_Component, _Colors>);
		app.update();

		let child = app
			.world
			.iter_entities()
			.find_map(|e| e.get::<_Child>())
			.unwrap();

		assert_eq!(_Colors::get_base_colors(), child.0);
	}
}
