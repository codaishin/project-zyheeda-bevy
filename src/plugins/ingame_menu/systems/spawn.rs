use crate::plugins::ingame_menu::traits::{
	children::Children,
	colors::HasBackgroundColor,
	spawn::Spawn,
};
use bevy::{
	ecs::{component::Component, system::Commands},
	hierarchy::BuildChildren,
	render::color::Color,
	ui::node_bundles::NodeBundle,
	utils::default,
};

pub fn spawn<TComponent: Spawn + Children + Component + HasBackgroundColor>(
	mut commands: Commands,
) {
	let (style, component) = TComponent::spawn();
	commands
		.spawn((
			NodeBundle {
				style,
				background_color: TComponent::BACKGROUND_COLOR.unwrap_or(Color::NONE).into(),
				..default()
			},
			component,
		))
		.with_children(|parent| TComponent::children(parent));
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		hierarchy::{ChildBuilder, Parent},
		prelude::default,
		render::color::Color,
		ui::{BackgroundColor, Style, Val},
	};

	#[derive(Component)]
	struct _Component;

	#[derive(Component)]
	struct _Child;

	impl Spawn for _Component {
		fn spawn() -> (Style, Self) {
			(
				Style {
					width: Val::Px(42.),
					..default()
				},
				Self,
			)
		}
	}

	impl Children for _Component {
		fn children(parent: &mut ChildBuilder) {
			parent.spawn(_Child);
		}
	}

	impl HasBackgroundColor for _Component {
		const BACKGROUND_COLOR: Option<Color> = Some(Color::rgb(0.1, 0.2, 0.3));
	}

	#[derive(Component)]
	struct _ComponentWithoutBackgroundColor;

	impl Spawn for _ComponentWithoutBackgroundColor {
		fn spawn() -> (Style, Self) {
			(Style::default(), Self)
		}
	}

	impl Children for _ComponentWithoutBackgroundColor {
		fn children(_parent: &mut ChildBuilder) {}
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
			.find(|e| e.contains::<_Component>());

		assert_eq!(
			(
				Some(&Style {
					width: Val::Px(42.),
					..default()
				}),
				Some(_Component::BACKGROUND_COLOR.unwrap())
			),
			(
				entity_with_component.and_then(|entity| entity.get::<Style>()),
				entity_with_component.and_then(|entity| entity
					.get::<BackgroundColor>()
					.map(|background_color| background_color.0))
			)
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

	#[test]
	fn spawn_children() {
		let mut app = App::new();

		app.add_systems(Update, spawn::<_Component>);
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
}
