use bevy::{mesh::MeshPlugin, prelude::*, scene::ScenePlugin};
use bevy_rapier3d::prelude::*;

pub(crate) struct TestCollisionsPlugin;

impl Plugin for TestCollisionsPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins((
			MinimalPlugins,
			TransformPlugin,
			AssetPlugin::default(),
			MeshPlugin,
			ScenePlugin,
			RapierPhysicsPlugin::<NoUserData>::default(),
		));
	}
}
