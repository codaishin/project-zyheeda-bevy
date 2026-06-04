use crate::traits::LoadUi;
use bevy::prelude::*;
use common::{traits::load_asset::LoadAsset, zyheeda_commands::ZyheedaCommands};

pub fn spawn<TComponent, TServer, TCameras>(
	mut commands: ZyheedaCommands,
	mut images: ResMut<TServer>,
	cameras: Query<Entity, With<TCameras>>,
) where
	TComponent: LoadUi<TServer> + Component,
	TServer: Resource + LoadAsset,
	TCameras: Component,
{
	let component = TComponent::load_ui(images.as_mut());

	let mut entity = commands.spawn(component);

	if let Ok(camera) = cameras.single() {
		entity.insert(UiTargetCamera(camera));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::asset::AssetPath;
	use testing::{SingleThreadedApp, assert_count};

	#[derive(Component, Resource, Default)]
	struct _Server;

	impl LoadAsset for _Server {
		fn load_asset<'a, TAsset, TPath>(&mut self, _: TPath) -> Handle<TAsset>
		where
			TAsset: Asset,
			TPath: Into<AssetPath<'a>>,
		{
			panic!("NOT USED")
		}
	}

	#[derive(Component)]
	struct _Component;

	impl LoadUi<_Server> for _Component {
		fn load_ui(_: &mut _Server) -> Self {
			_Component
		}
	}

	#[derive(Component)]
	struct _Camera;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<_Server>();
		app.add_systems(Update, spawn::<_Component, _Server, _Camera>);

		app
	}

	#[test]
	fn spawn_component() {
		let mut app = setup();

		app.update();

		let mut components = app.world_mut().query_filtered::<(), With<_Component>>();
		assert_count!(1, components.iter(app.world()));
	}

	#[test]
	fn set_ui_target_camera() {
		let mut app = setup();
		let camera = app.world_mut().spawn(_Camera).id();

		app.update();

		let mut components = app
			.world_mut()
			.query_filtered::<&UiTargetCamera, With<_Component>>();
		let [UiTargetCamera(target_camera)] = assert_count!(1, components.iter(app.world()));
		assert_eq!(&camera, target_camera);
	}
}
