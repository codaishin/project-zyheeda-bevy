use crate::traits::LoadUi;
use bevy::prelude::*;
use common::traits::{handles_graphics::StaticRenderLayers, load_asset::LoadAsset};

pub fn spawn<TComponent, TServer, TGraphics>(mut commands: Commands, mut images: ResMut<TServer>)
where
	TComponent: LoadUi<TServer> + Component,
	TServer: Resource + LoadAsset,
	TGraphics: StaticRenderLayers,
{
	let component = TComponent::load_ui(images.as_mut());

	commands.spawn((component, TGraphics::render_layers()));
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{asset::AssetPath, render::view::RenderLayers};

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

	struct _Graphics;

	impl StaticRenderLayers for _Graphics {
		fn render_layers() -> RenderLayers {
			RenderLayers::layer(11)
		}
	}

	#[test]
	fn spawn_component() {
		let mut app = App::new();

		app.init_resource::<_Server>();
		app.add_systems(Update, spawn::<_Component, _Server, _Graphics>);
		app.update();

		assert_eq!(
			1,
			app.world()
				.iter_entities()
				.filter(|e| e.contains::<_Component>())
				.count()
		);
	}

	#[test]
	fn spawn_render_layer() {
		let mut app = App::new();

		app.init_resource::<_Server>();
		app.add_systems(Update, spawn::<_Component, _Server, _Graphics>);
		app.update();

		assert_eq!(
			1,
			app.world()
				.iter_entities()
				.filter_map(|e| e.get::<RenderLayers>())
				.filter(|r| r == &&RenderLayers::layer(11))
				.count()
		);
	}
}
