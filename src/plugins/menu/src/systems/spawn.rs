use crate::traits::LoadUi;
use bevy::prelude::*;
use common::traits::load_asset::LoadAsset;

pub fn spawn<TComponent: LoadUi<TServer> + Component, TServer: Resource + LoadAsset>(
	mut commands: Commands,
	mut images: ResMut<TServer>,
) {
	let component = TComponent::load_ui(images.as_mut());

	commands.spawn(component);
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::asset::AssetPath;

	#[derive(Component, Resource, Default)]
	struct _Server;

	impl LoadAsset for _Server {
		fn load_asset<TAsset, TPath>(&mut self, _: TPath) -> Handle<TAsset>
		where
			TAsset: Asset,
			TPath: Into<AssetPath<'static>> + 'static,
		{
			Handle::default()
		}
	}

	#[derive(Component)]
	struct _Component;

	impl LoadUi<_Server> for _Component {
		fn load_ui(_: &mut _Server) -> Self {
			_Component
		}
	}

	#[test]
	fn spawn_bundle() {
		let mut app = App::new();

		app.init_resource::<_Server>();
		app.add_systems(Update, spawn::<_Component, _Server>);
		app.update();

		assert_eq!(
			1,
			app.world()
				.iter_entities()
				.filter(|e| e.contains::<_Component>())
				.count()
		);
	}
}
