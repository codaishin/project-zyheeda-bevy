use crate::components::{child_meshes::ChildMeshes, pass_layer::PassLayers};
use bevy::{camera::visibility::RenderLayers, prelude::*};
use common::{traits::accessors::get::TryApplyOn, zyheeda_commands::ZyheedaCommands};

type PassLayerOrChildMeshesChanged = Or<(Changed<PassLayers>, Changed<ChildMeshes>)>;

impl PassLayers {
	pub(crate) fn propagate_layer(
		mut commands: ZyheedaCommands,
		layers: Query<(Entity, &PassLayers, Option<&ChildMeshes>), PassLayerOrChildMeshesChanged>,
	) {
		for (root, pass_layers, children) in layers {
			for entity in iter(root, children) {
				commands.try_apply_on(&entity, |mut e| {
					e.try_insert(RenderLayers::from_iter(pass_layers));
				});
			}
		}
	}
}

fn iter(root: Entity, children: Option<&ChildMeshes>) -> impl Iterator<Item = Entity> {
	std::iter::once(root).chain(children.into_iter().flat_map(ChildMeshes::iter))
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::child_meshes::ChildMeshOf;
	use bevy::camera::visibility::RenderLayers;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, PassLayers::propagate_layer);

		app
	}

	#[test]
	fn set_layer_on_root() {
		let mut app = setup();
		let entity = app.world_mut().spawn(PassLayers::from(2)).id();

		app.update();

		assert_eq!(
			Some(&RenderLayers::layer(2)),
			app.world().entity(entity).get::<RenderLayers>(),
		);
	}

	#[test]
	fn set_layer_on_child() {
		let mut app = setup();
		let parent = app.world_mut().spawn(PassLayers::from(2)).id();
		let child = app.world_mut().spawn(ChildMeshOf(parent)).id();

		app.update();

		assert_eq!(
			Some(&RenderLayers::layer(2)),
			app.world().entity(child).get::<RenderLayers>(),
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();
		let parent = app.world_mut().spawn(PassLayers::from(2)).id();
		let child = app.world_mut().spawn(ChildMeshOf(parent)).id();

		app.update();
		app.world_mut().entity_mut(parent).remove::<RenderLayers>();
		app.world_mut().entity_mut(child).remove::<RenderLayers>();
		app.update();

		assert_eq!(
			(None, None),
			(
				app.world().entity(parent).get::<RenderLayers>(),
				app.world().entity(child).get::<RenderLayers>(),
			)
		);
	}

	#[test]
	fn act_again_if_pass_layer_changed() {
		let mut app = setup();
		let parent = app.world_mut().spawn(PassLayers::from(2)).id();
		let child = app.world_mut().spawn(ChildMeshOf(parent)).id();

		app.update();
		app.world_mut()
			.entity_mut(parent)
			.remove::<RenderLayers>()
			.get_mut::<PassLayers>()
			.as_deref_mut();
		app.world_mut().entity_mut(child).remove::<RenderLayers>();
		app.update();

		assert_eq!(
			(Some(&RenderLayers::layer(2)), Some(&RenderLayers::layer(2))),
			(
				app.world().entity(parent).get::<RenderLayers>(),
				app.world().entity(child).get::<RenderLayers>(),
			)
		);
	}

	#[test]
	fn act_again_if_children_changed() {
		let mut app = setup();
		let parent = app.world_mut().spawn(PassLayers::from(2)).id();
		let child = app.world_mut().spawn(ChildMeshOf(parent)).id();

		app.update();
		app.world_mut()
			.entity_mut(parent)
			.remove::<RenderLayers>()
			.get_mut::<ChildMeshes>()
			.as_deref_mut();
		app.world_mut().entity_mut(child).remove::<RenderLayers>();
		app.update();

		assert_eq!(
			(Some(&RenderLayers::layer(2)), Some(&RenderLayers::layer(2))),
			(
				app.world().entity(parent).get::<RenderLayers>(),
				app.world().entity(child).get::<RenderLayers>(),
			)
		);
	}
}
