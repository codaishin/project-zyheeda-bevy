use crate::traits::LoadUi;
use bevy::{
	ecs::{component::Mutable, system::StaticSystemParam},
	prelude::*,
};
use common::{
	traits::{
		accessors::get::GetContextMut,
		handles_graphics::{CameraHandle, RenderUi},
		load_asset::LoadAsset,
	},
	zyheeda_commands::ZyheedaCommands,
};

pub fn spawn<TComponent, TServer, TCamera>(
	mut commands: ZyheedaCommands,
	mut images: ResMut<TServer>,
	mut cameras: StaticSystemParam<TCamera>,
) where
	TComponent: LoadUi<TServer> + Component,
	TServer: Resource<Mutability = Mutable> + LoadAsset,
	TCamera: for<'c> GetContextMut<CameraHandle, TContext<'c>: RenderUi>,
{
	let ui = TComponent::load_ui(images.as_mut());
	let mut camera = TCamera::get_context_mut(&mut cameras, CameraHandle);

	let ui = commands.spawn(ui).id();
	camera.render_ui(ui);
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::asset::AssetPath;
	use testing::{SingleThreadedApp, assert_count};

	#[derive(Resource, Default)]
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

	#[derive(Resource, Default)]
	struct _Camera {
		renders: Vec<Entity>,
	}

	impl RenderUi for &mut _Camera {
		fn render_ui(&mut self, ui: Entity) {
			self.renders.push(ui);
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<_Server>();
		app.init_resource::<_Camera>();
		app.add_systems(Update, spawn::<_Component, _Server, ResMut<_Camera>>);

		app
	}

	#[test]
	fn spawn_component() {
		let mut app = setup();

		app.update();

		let mut uis = app.world_mut().query_filtered::<(), With<_Component>>();
		assert_count!(1, uis.iter(app.world()));
	}

	#[test]
	fn render_ui() {
		let mut app = setup();

		app.update();

		let mut uis = app.world_mut().query_filtered::<Entity, With<_Component>>();
		let [ui] = assert_count!(1, uis.iter(app.world()));
		assert_eq!(vec![ui], app.world().resource::<_Camera>().renders);
	}
}
