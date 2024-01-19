use crate::{
	components::{SimpleMovement, WaitNext},
	resources::ModelData,
	traits::{model::Model, projectile_behavior::ProjectileBehavior},
};
use bevy::{
	asset::Handle,
	ecs::{
		component::Component,
		entity::Entity,
		query::{Added, With},
		system::{Commands, EntityCommands, Query, Res},
	},
	hierarchy::{BuildChildren, DespawnRecursiveExt},
	math::Vec3,
	pbr::{PbrBundle, StandardMaterial},
	render::mesh::Mesh,
	transform::components::GlobalTransform,
	utils::default,
};

pub fn projectile<TProjectile: Model<StandardMaterial> + ProjectileBehavior + Component>(
	mut commands: Commands,
	mode_data: Res<ModelData<StandardMaterial, TProjectile>>,
	projectiles: Query<(Entity, &TProjectile, &GlobalTransform), Added<TProjectile>>,
	waiting: Query<Entity, (With<WaitNext>, With<TProjectile>)>,
) {
	for entity in &waiting {
		commands.entity(entity).despawn_recursive();
	}

	if projectiles.is_empty() {
		return;
	}

	for (id, projectile, transform) in &projectiles {
		let target = get_target(projectile, transform);
		let model = get_model(&mut commands, &mode_data.material, &mode_data.mesh);
		let entity = &mut commands.entity(id);
		configure(entity, target, model);
	}
}

fn get_target<TProjectile: ProjectileBehavior>(
	projectile: &TProjectile,
	transform: &GlobalTransform,
) -> Vec3 {
	transform.translation() + projectile.direction() * projectile.range()
}

fn get_model(
	commands: &mut Commands,
	material: &Handle<StandardMaterial>,
	mesh: &Handle<Mesh>,
) -> Entity {
	let model = commands
		.spawn(PbrBundle {
			material: material.clone(),
			mesh: mesh.clone(),
			..default()
		})
		.id();
	model
}

fn configure(entity: &mut EntityCommands, target: Vec3, model: Entity) {
	entity.insert(SimpleMovement { target }).add_child(model);
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{SimpleMovement, WaitNext},
		test_tools::utils::GetImmediateChildren,
	};
	use bevy::{
		app::{App, Update},
		asset::{AssetId, Handle},
		ecs::component::Component,
		math::Vec3,
		pbr::StandardMaterial,
		render::{
			color::Color,
			mesh::{shape, Mesh},
		},
		transform::components::Transform,
		utils::{default, Uuid},
	};

	#[derive(Component, Default)]
	struct _Projectile {
		pub direction: Vec3,
		pub range: f32,
	}

	impl ProjectileBehavior for _Projectile {
		fn direction(&self) -> bevy::prelude::Vec3 {
			self.direction
		}
		fn range(&self) -> f32 {
			self.range
		}
	}

	impl Model<StandardMaterial> for _Projectile {
		fn material() -> StandardMaterial {
			StandardMaterial {
				base_color: Color::RED,
				..default()
			}
		}
		fn mesh() -> Mesh {
			shape::Icosphere {
				radius: 42.,
				subdivisions: 5,
			}
			.try_into()
			.unwrap()
		}
	}

	fn setup(model_data: ModelData<StandardMaterial, _Projectile>) -> App {
		let mut app = App::new();
		app.insert_resource(model_data);
		app.add_systems(Update, projectile::<_Projectile>);

		app
	}

	#[test]
	fn spawn_with_material() {
		let material = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let mut app = setup(ModelData::new(material.clone(), default()));

		let projectile = app
			.world
			.spawn((
				_Projectile { ..default() },
				GlobalTransform::from_translation(Vec3::ZERO),
			))
			.id();

		app.update();

		let projectile_materials =
			Handle::<StandardMaterial>::get_immediate_children(&projectile, &app);
		let projectile_material = projectile_materials.first();

		assert_eq!(Some(&&material), projectile_material);
	}

	#[test]
	fn spawn_with_mesh() {
		let mesh = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let mut app = setup(ModelData::new(default(), mesh.clone()));

		let projectile = app
			.world
			.spawn((
				_Projectile { ..default() },
				GlobalTransform::from_translation(Vec3::ZERO),
			))
			.id();

		app.update();

		let projectile_meshes = Handle::<Mesh>::get_immediate_children(&projectile, &app);
		let projectile_mesh = projectile_meshes.first();

		assert_eq!(Some(&&mesh), projectile_mesh);
	}

	#[test]
	fn spawn_with_simple_movement() {
		let mut app = setup(default());

		let projectile = app
			.world
			.spawn((
				_Projectile {
					direction: Vec3::new(1., 2., 3.),
					range: 42.,
				},
				GlobalTransform::from_translation(Vec3::ZERO),
			))
			.id();

		app.update();

		let projectile = app.world.entity(projectile);

		assert_eq!(
			Some(&SimpleMovement {
				target: Vec3::new(1., 2., 3.) * 42.
			}),
			projectile.get::<SimpleMovement>()
		);
	}

	#[test]
	fn spawn_with_simple_movement_from_offset() {
		let mut app = setup(default());

		let projectile = app
			.world
			.spawn((
				_Projectile {
					direction: Vec3::new(1., 2., 3.),
					range: 42.,
				},
				GlobalTransform::from_translation(Vec3::new(10., 20., 30.)),
			))
			.id();

		app.update();

		let projectile = app.world.entity(projectile);

		assert_eq!(
			Some(&SimpleMovement {
				target: Vec3::new(10., 20., 30.) + Vec3::new(1., 2., 3.) * 42.
			}),
			projectile.get::<SimpleMovement>()
		);
	}

	#[test]
	fn despawn_when_wait_next_added() {
		let mut app = setup(default());

		let projectile = app
			.world
			.spawn((
				_Projectile {
					direction: Vec3::new(1., 2., 3.),
					range: 42.,
				},
				GlobalTransform::from_translation(Vec3::ZERO),
			))
			.id();

		app.update();

		app.world.entity_mut(projectile).insert(WaitNext);

		app.update();

		assert_eq!(
			0,
			app.world
				.iter_entities()
				.filter(|entity| entity.contains::<Handle<Mesh>>()
					|| entity.contains::<Handle<StandardMaterial>>()
					|| entity.contains::<SimpleMovement>())
				.count()
		);
	}

	#[test]
	fn do_not_despawn_when_projectile_missing() {
		#[derive(Component)]
		struct _Decoy;

		let mut app = setup(default());

		app.world.spawn((_Decoy, WaitNext));
		app.update();

		assert_eq!(
			1,
			app.world
				.iter_entities()
				.filter(|entity| entity.contains::<_Decoy>())
				.count()
		);
	}

	#[test]
	fn spawn_only_one_child() {
		let mut app = setup(default());

		let projectile = app
			.world
			.spawn((
				_Projectile { ..default() },
				GlobalTransform::from_translation(Vec3::ZERO),
			))
			.id();

		app.update();
		app.update();

		let child_count = Transform::get_immediate_children(&projectile, &app).len();

		assert_eq!(1, child_count);
	}
}
