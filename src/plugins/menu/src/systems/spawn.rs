use crate::traits::{
	ui_components::{GetUIComponents, GetZIndex, GetZIndexGlobal},
	LoadUi,
};
use bevy::prelude::*;
use common::traits::load_asset::LoadAsset;

pub fn spawn<
	TComponent: LoadUi<TServer> + GetUIComponents + GetZIndex + GetZIndexGlobal + Component,
	TServer: Resource + LoadAsset,
>(
	mut commands: Commands,
	mut images: ResMut<TServer>,
) {
	let component = TComponent::load_ui(images.as_mut());
	let z_index = component.z_index();
	let z_index_global = component.z_index_global();

	let mut entity = commands.spawn((component.ui_components(), component));

	if let Some(z_index) = z_index {
		entity.insert(z_index);
	}

	if let Some(z_index_global) = z_index_global {
		entity.insert(z_index_global);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{asset::AssetPath, color::palettes::tailwind::CYAN_50};

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

	#[derive(Component)]
	struct _Child;

	impl GetZIndex for _Component {
		fn z_index(&self) -> Option<ZIndex> {
			Some(ZIndex(42))
		}
	}

	impl GetZIndexGlobal for _Component {
		fn z_index_global(&self) -> Option<GlobalZIndex> {
			Some(GlobalZIndex(420))
		}
	}

	impl GetUIComponents for _Component {
		fn ui_components(&self) -> (Node, BackgroundColor) {
			(
				Node {
					width: Val::Px(42.),
					..default()
				},
				BackgroundColor(CYAN_50.into()),
			)
		}
	}

	#[test]
	fn spawn_bundle() {
		let mut app = App::new();

		app.init_resource::<_Server>();
		app.add_systems(Update, spawn::<_Component, _Server>);
		app.update();

		let entity = app
			.world()
			.iter_entities()
			.find(|e| e.contains::<_Component>())
			.expect("no _Component spawned");
		assert_eq!(
			(
				Some(&Node {
					width: Val::Px(42.),
					..default()
				}),
				Some(&BackgroundColor(CYAN_50.into())),
				Some(&ZIndex(42)),
				Some(&GlobalZIndex(420)),
			),
			(
				entity.get::<Node>(),
				entity.get::<BackgroundColor>(),
				entity.get::<ZIndex>(),
				entity.get::<GlobalZIndex>(),
			)
		);
	}
}
