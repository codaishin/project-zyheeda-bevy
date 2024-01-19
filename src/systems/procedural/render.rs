use crate::resources::ModelData;
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
};

pub fn render<TComponent: Component>(
	mut commands: Commands,
	agents: Query<Entity, Added<TComponent>>,
	model_data: Res<ModelData<StandardMaterial, TComponent>>,
) {
	for agent in &agents {
		let model = commands
			.spawn(PbrBundle {
				material: model_data.material.clone(),
				mesh: model_data.mesh.clone(),
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
	use crate::test_tools::utils::GetImmediateChildren;
	use bevy::{
		app::{App, Update},
		asset::{AssetId, Handle},
		render::mesh::Mesh,
		transform::components::Transform,
		utils::{default, Uuid},
	};

	#[derive(Component)]
	struct _Model;

	fn setup(model_data: ModelData<StandardMaterial, _Model>) -> App {
		let mut app = App::new();
		app.insert_resource(model_data);
		app.add_systems(Update, render::<_Model>);

		app
	}

	#[test]
	fn spawn_material() {
		let material = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let mut app = setup(ModelData::new(material.clone(), default()));

		let agent = app.world.spawn(_Model).id();
		app.update();

		let agent_materials = Handle::<StandardMaterial>::get_immediate_children(&agent, &app);
		let agent_material = agent_materials.first();

		assert_eq!(Some(&&material), agent_material);
	}

	#[test]
	fn spawn_mesh() {
		let mesh = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let mut app = setup(ModelData::new(default(), mesh.clone()));

		let agent = app.world.spawn(_Model).id();
		app.update();

		let agent_meshes = Handle::<Mesh>::get_immediate_children(&agent, &app);
		let agent_mesh = agent_meshes.first();

		assert_eq!(Some(&&mesh), agent_mesh);
	}

	#[test]
	fn spawn_only_one_child() {
		let mut app = setup(ModelData::new(default(), default()));

		let agent = app.world.spawn(_Model).id();
		app.update();
		app.update();

		let child_count = Transform::get_immediate_children(&agent, &app).len();

		assert_eq!(1, child_count);
	}
}
