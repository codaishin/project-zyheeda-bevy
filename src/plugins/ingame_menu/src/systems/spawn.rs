use crate::traits::{colors::HasBackgroundColor, get_style::GetStyle};
use bevy::{
	ecs::{component::Component, system::Commands},
	render::color::Color,
	ui::node_bundles::NodeBundle,
	utils::default,
};

pub fn spawn<TComponent: Default + GetStyle + Component + HasBackgroundColor>(
	mut commands: Commands,
) {
	let component = TComponent::default();
	commands.spawn((
		NodeBundle {
			style: component.style(),
			background_color: TComponent::BACKGROUND_COLOR.unwrap_or(Color::NONE).into(),
			..default()
		},
		component,
	));
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		prelude::default,
		render::color::Color,
		ui::{BackgroundColor, Style, Val},
	};

	#[derive(Component, Default)]
	struct _Component;

	#[derive(Component)]
	struct _Child;

	impl GetStyle for _Component {
		fn style(&self) -> Style {
			Style {
				width: Val::Px(42.),
				..default()
			}
		}
	}

	impl HasBackgroundColor for _Component {
		const BACKGROUND_COLOR: Option<Color> = Some(Color::rgb(0.1, 0.2, 0.3));
	}

	#[derive(Component, Default)]
	struct _ComponentWithoutBackgroundColor;

	impl GetStyle for _ComponentWithoutBackgroundColor {
		fn style(&self) -> Style {
			Style::default()
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
}
