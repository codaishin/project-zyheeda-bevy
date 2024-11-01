use bevy::prelude::*;

pub trait InitResource
where
	Self: Resource + Default,
{
	fn init(mut commands: Commands) {
		commands.init_resource::<Self>();
	}
}

impl<T> InitResource for T where T: Resource + Default {}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::RunSystemOnce;

	#[derive(Resource, Default, Debug, PartialEq)]
	struct _Resource;

	fn setup() -> App {
		App::new()
	}

	#[test]
	fn init_resource() {
		let mut app = setup();

		app.world_mut().run_system_once(_Resource::init);

		assert_eq!(Some(&_Resource), app.world().get_resource::<_Resource>());
	}
}
