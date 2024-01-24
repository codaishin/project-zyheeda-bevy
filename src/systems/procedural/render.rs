use crate::{resources::ModelData, traits::model::Offset};
use bevy::{
	ecs::{
		component::Component,
		entity::Entity,
		query::Added,
		system::{Commands, Query, Res},
	},
	hierarchy::BuildChildren,
	pbr::{PbrBundle, StandardMaterial},
	prelude::default,
	transform::components::Transform,
};

pub fn render<TComponent: Component + Offset>(
	mut commands: Commands,
	agents: Query<Entity, Added<TComponent>>,
	model_data: Res<ModelData<StandardMaterial, TComponent>>,
) {
	for agent in &agents {
		let model = commands
			.spawn(PbrBundle {
				material: model_data.material.clone(),
				mesh: model_data.mesh.clone(),
				transform: Transform::from_translation(TComponent::offset()),
				..default()
			})
			.id();
		let mut agent = commands.entity(agent);
		agent.add_child(model);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::test_tools::utils::{GetImmediateChildComponents, GetImmediateChildren};
	use bevy::{
		app::{App, Update},
		asset::{AssetId, Handle},
		math::Vec3,
		render::mesh::Mesh,
		transform::components::Transform,
		utils::{default, Uuid},
	};

	#[derive(Component)]
	struct _Model;

	impl Offset for _Model {
		fn offset() -> Vec3 {
			Vec3::new(1., 2., 3.)
		}
	}

	fn setup(model_data: ModelData<StandardMaterial, _Model>) -> App {
		let mut app = App::new();
		app.insert_resource(model_data);
		app.add_systems(Update, render::<_Model>);

		app
	}

	#[test]
	fn spawn_material() {
		let orig_material = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let mut app = setup(ModelData::new(orig_material.clone(), default()));

		let agent = app.world.spawn(_Model).id();
		app.update();

		let materials = Handle::<StandardMaterial>::get_immediate_children(&agent, &app);
		let material = materials.first();

		assert_eq!(Some(&&orig_material), material);
	}

	#[test]
	fn spawn_mesh() {
		let orig_mesh = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let mut app = setup(ModelData::new(default(), orig_mesh.clone()));

		let agent = app.world.spawn(_Model).id();
		app.update();

		let meshes = Handle::<Mesh>::get_immediate_children(&agent, &app);
		let mesh = meshes.first();

		assert_eq!(Some(&&orig_mesh), mesh);
	}

	#[test]
	fn spawn_mesh_with_offset() {
		let mesh = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let mut app = setup(ModelData::new(default(), mesh.clone()));

		let agent = app.world.spawn(_Model).id();
		app.update();

		let transforms = Transform::get_immediate_children(&agent, &app);
		let transform = transforms.first();

		assert_eq!(Some(&&Transform::from_xyz(1., 2., 3.)), transform);
	}

	#[test]
	fn spawn_only_one_child() {
		let mut app = setup(ModelData::new(default(), default()));

		let agent = app.world.spawn(_Model).id();
		app.update();
		app.update();

		let children = Entity::get_immediate_children(&agent, &app);

		assert_eq!(1, children.len());
	}
}
