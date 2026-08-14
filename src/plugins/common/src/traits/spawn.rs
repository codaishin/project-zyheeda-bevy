use bevy::prelude::*;

pub trait SpawnDefault: Component {
	fn spawn_default(commands: Commands);
}

impl<T> SpawnDefault for T
where
	T: Component + Default,
{
	fn spawn_default(mut commands: Commands) {
		commands.spawn(Self::default());
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use testing::{SingleThreadedApp, assert_count};

	#[derive(Component, Debug, PartialEq, Default)]
	struct _Component;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, _Component::spawn_default);

		app
	}

	#[test]
	fn spawn_component() {
		let mut app = setup();

		app.update();

		let mut entities_with_component = app.world_mut().query::<&_Component>();
		assert_count!(1, entities_with_component.iter(app.world()));
	}
}
