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
	use bevy::{asset::AssetPath, camera::visibility::RenderLayers};
	use testing::assert_count;

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

		let mut components = app.world_mut().query_filtered::<(), With<_Component>>();
		assert_count!(1, components.iter(app.world()));
	}

	#[test]
	fn spawn_render_layer() {
		let mut app = App::new();

		app.init_resource::<_Server>();
		app.add_systems(Update, spawn::<_Component, _Server, _Graphics>);
		app.update();

		let mut render_layers = app.world_mut().query::<&RenderLayers>();
		assert_count!(
			1,
			render_layers
				.iter(app.world())
				.filter(|r| r == &&RenderLayers::layer(11))
		);
	}
}
