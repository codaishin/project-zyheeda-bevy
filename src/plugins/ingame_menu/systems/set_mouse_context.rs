use crate::states::MouseContext;
use bevy::{
	ecs::{
		schedule::NextState,
		system::{Query, ResMut},
	},
	ui::Interaction,
};

pub fn set_mouse_context(
	mut state: ResMut<NextState<MouseContext>>,
	interactions: Query<&Interaction>,
) {
	let context = match interactions.iter().any(|i| *i != Interaction::None) {
		true => MouseContext::UI,
		false => MouseContext::Default,
	};
	state.set(context);
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::states::MouseContext;
	use bevy::{
		app::{App, Update},
		ecs::schedule::State,
		ui::Interaction,
	};

	#[test]
	fn set_context_ui_when_some_interaction_hovered() {
		let mut app = App::new();

		app.add_systems(Update, set_mouse_context);
		app.add_state::<MouseContext>();
		app.world.spawn(Interaction::Hovered);
		app.world
			.resource_mut::<NextState<MouseContext>>()
			.set(MouseContext::Default);

		app.update();
		app.update();

		assert_eq!(
			&MouseContext::UI,
			app.world
				.get_resource::<State<MouseContext>>()
				.unwrap()
				.get()
		);
	}

	#[test]
	fn set_context_ui_when_some_interaction_pressed() {
		let mut app = App::new();

		app.add_systems(Update, set_mouse_context);
		app.add_state::<MouseContext>();
		app.world.spawn(Interaction::Pressed);
		app.world
			.resource_mut::<NextState<MouseContext>>()
			.set(MouseContext::Default);

		app.update();
		app.update();

		assert_eq!(
			&MouseContext::UI,
			app.world
				.get_resource::<State<MouseContext>>()
				.unwrap()
				.get()
		);
	}

	#[test]
	fn set_context_default_when_no_interaction() {
		let mut app = App::new();

		app.add_systems(Update, set_mouse_context);
		app.add_state::<MouseContext>();
		app.world.spawn(Interaction::None);
		app.world
			.resource_mut::<NextState<MouseContext>>()
			.set(MouseContext::UI);

		app.update();
		app.update();

		assert_eq!(
			&MouseContext::Default,
			app.world
				.get_resource::<State<MouseContext>>()
				.unwrap()
				.get()
		);
	}
}
