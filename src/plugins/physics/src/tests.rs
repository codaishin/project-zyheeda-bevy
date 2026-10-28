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
		app.configure_sets(
			Update,
			AfterTestPhysicsBackend
				.after(PhysicsSet::SyncBackend)
				.after(PhysicsSet::StepSimulation)
				.after(PhysicsSet::Writeback),
		);
	}
}

#[derive(SystemSet, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) struct AfterTestPhysicsBackend;
