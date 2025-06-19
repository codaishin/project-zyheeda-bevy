use bevy::prelude::*;

pub trait Spawn: Component {
	fn spawn(commands: Commands);
}

impl<T> Spawn for T
where
	T: Component + Default,
{
	fn spawn(mut commands: Commands) {
		commands.spawn(Self::default());
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{assert_count, test_tools::utils::SingleThreadedApp};

	#[derive(Component, Debug, PartialEq, Default)]
	struct _Component;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, _Component::spawn);

		app
	}

	#[test]
	fn spawn_component() {
		let mut app = setup();

		app.update();

		let entities_with_component = app
			.world()
			.iter_entities()
			.filter(|e| e.contains::<_Component>());
		assert_count!(1, entities_with_component);
	}
}
