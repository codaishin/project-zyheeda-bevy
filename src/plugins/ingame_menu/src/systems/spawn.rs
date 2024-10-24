use crate::traits::{get_node::GetNode, LoadUi};
use bevy::{
	ecs::{component::Component, system::Commands},
	prelude::{ResMut, Resource},
};
use common::traits::load_asset::LoadAsset;

pub fn spawn<TComponent: LoadUi<TServer> + GetNode + Component, TServer: Resource + LoadAsset>(
	mut commands: Commands,
	mut images: ResMut<TServer>,
) {
	let component = TComponent::load_ui(images.as_mut());
	commands.spawn((component.node(), component));
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		asset::{Asset, AssetPath, Handle},
		prelude::default,
		ui::{node_bundles::NodeBundle, Style, Val},
	};
	use common::assert_bundle;

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

	impl GetNode for _Component {
		fn node(&self) -> NodeBundle {
			NodeBundle {
				style: Style {
					width: Val::Px(42.),
					..default()
				},
				..default()
			}
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

		assert_bundle!(
			NodeBundle,
			&app,
			entity,
			With::assert(|style| assert_eq!(
				&Style {
					width: Val::Px(42.),
					..default()
				},
				style
			))
		);
	}
}
