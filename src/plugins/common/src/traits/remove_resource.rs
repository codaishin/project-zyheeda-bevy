use bevy::prelude::*;

pub trait RemoveResource
where
	Self: Resource + Sized,
{
	fn remove(mut commands: Commands) {
		commands.remove_resource::<Self>();
	}
}

impl<T> RemoveResource for T where T: Resource {}

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
	fn remove_resource() {
		let mut app = setup();
		app.init_resource::<_Resource>();

		app.world_mut().run_system_once(_Resource::remove);

		assert_eq!(None, app.world().get_resource::<_Resource>());
	}
}
