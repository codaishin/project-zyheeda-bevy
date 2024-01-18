use crate::{
	components::{SimpleMovement, WaitNext},
	traits::projectile::{ProjectileBehaviorData, ProjectileModelData},
};
use bevy::{
	asset::{Assets, Handle},
	ecs::{
		component::Component,
		entity::Entity,
		query::{Added, With},
		system::{Commands, EntityCommands, Local, Query, ResMut},
	},
	hierarchy::{BuildChildren, DespawnRecursiveExt},
	math::Vec3,
	pbr::{PbrBundle, StandardMaterial},
	render::mesh::Mesh,
	transform::components::GlobalTransform,
	utils::default,
};

pub fn projectile<TProjectile: ProjectileModelData + ProjectileBehaviorData + Component>(
	mut commands: Commands,
	mut materials: ResMut<Assets<StandardMaterial>>,
	mut meshes: ResMut<Assets<Mesh>>,
	mut local: Local<Option<(Handle<StandardMaterial>, Handle<Mesh>)>>,
	projectiles: Query<(Entity, &TProjectile, &GlobalTransform), Added<TProjectile>>,
	waiting: Query<Entity, With<WaitNext>>,
) {
	for entity in &waiting {
		commands.entity(entity).despawn_recursive();
	}

	if projectiles.is_empty() {
		return;
	}

	let (material, mesh) = local.get_or_insert_with(|| {
		(
			materials.add(TProjectile::material()),
			meshes.add(TProjectile::mesh()),
		)
	});

	for (id, projectile, transform) in &projectiles {
		let target = get_target(projectile, transform);
		let model = get_model(&mut commands, material, mesh);
		let entity = &mut commands.entity(id);
		configure(entity, target, model);
	}
}

fn get_target<TProjectile: ProjectileBehaviorData>(
	projectile: &TProjectile,
	transform: &GlobalTransform,
) -> Vec3 {
	transform.translation() + projectile.direction() * projectile.range()
}

fn get_model(
	commands: &mut Commands,
	material: &mut Handle<StandardMaterial>,
	mesh: &mut Handle<Mesh>,
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
	use crate::components::{SimpleMovement, WaitNext};
	use bevy::{
		app::{App, Update},
		asset::{Asset, AssetId, Assets, Handle},
		ecs::component::Component,
		hierarchy::Children,
		math::Vec3,
		pbr::StandardMaterial,
		render::{
			color::Color,
			mesh::{shape, Mesh},
		},
		transform::components::Transform,
		utils::default,
	};

	#[derive(Component, Default)]
	struct _Projectile {
		pub direction: Vec3,
		pub range: f32,
	}

	impl ProjectileBehaviorData for _Projectile {
		fn direction(&self) -> bevy::prelude::Vec3 {
			self.direction
		}
		fn range(&self) -> f32 {
			self.range
		}
	}

	impl ProjectileModelData for _Projectile {
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

	fn setup() -> App {
		let mut app = App::new();
		app.insert_resource(Assets::<StandardMaterial>::default());
		app.insert_resource(Assets::<Mesh>::default());
		app.add_systems(Update, projectile::<_Projectile>);

		app
	}

	fn get_component_in_children<'a, TComponent: Component>(
		entity: &Entity,
		app: &'a App,
	) -> Vec<&'a TComponent> {
		match app.world.entity(*entity).get::<Children>() {
			None => vec![],
			Some(children) => children
				.iter()
				.filter_map(|entity| app.world.entity(*entity).get::<TComponent>())
				.collect(),
		}
	}

	fn get_asset<'a, TAsset: Asset>(seek: &AssetId<TAsset>, app: &'a App) -> Option<&'a TAsset> {
		let assets = app.world.resource::<Assets<TAsset>>();
		let assets: Vec<_> = assets.iter().collect();
		assets
			.iter()
			.find_map(|(id, asset)| if id == seek { Some(asset) } else { None })
			.cloned()
	}

	#[test]
	fn spawn_with_material() {
		let mut app = setup();

		let projectile = app
			.world
			.spawn((
				_Projectile { ..default() },
				GlobalTransform::from_translation(Vec3::ZERO),
			))
			.id();

		app.update();

		let material = get_component_in_children::<Handle<StandardMaterial>>(&projectile, &app)
			.first()
			.and_then(|handle| get_asset(&handle.id(), &app));

		assert_eq!(
			Some(_Projectile::material().base_color),
			material.map(|m| m.base_color)
		);
	}

	#[test]
	fn spawn_with_mesh() {
		let mut app = setup();

		let projectile = app
			.world
			.spawn((
				_Projectile { ..default() },
				GlobalTransform::from_translation(Vec3::ZERO),
			))
			.id();

		app.update();

		let mesh = get_component_in_children::<Handle<Mesh>>(&projectile, &app)
			.first()
			.and_then(|handle| get_asset(&handle.id(), &app));

		assert_eq!(
			Some(_Projectile::mesh().primitive_topology()),
			mesh.map(|mesh| mesh.primitive_topology())
		);
	}

	#[test]
	fn spawn_with_simple_movement() {
		let mut app = setup();

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
		let mut app = setup();

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
		let mut app = setup();

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
	fn register_meshes_and_materials_only_once() {
		let mut app = setup();

		app.world.spawn((
			_Projectile {
				direction: Vec3::new(1., 2., 3.),
				range: 42.,
			},
			GlobalTransform::from_translation(Vec3::ZERO),
		));

		app.update();
		app.update();

		let mesh_count = app.world.resource::<Assets<Mesh>>().iter().count();
		let material_count = app
			.world
			.resource::<Assets<StandardMaterial>>()
			.iter()
			.count();

		assert_eq!((1, 1), (mesh_count, material_count));
	}

	#[test]
	fn forgo_registering_meshes_and_materials_when_no_projectile_ever_present() {
		let mut app = setup();

		app.update();

		let mesh_count = app.world.resource::<Assets<Mesh>>().iter().count();
		let material_count = app
			.world
			.resource::<Assets<StandardMaterial>>()
			.iter()
			.count();

		assert_eq!((0, 0), (mesh_count, material_count));
	}

	#[test]
	fn spawn_only_one_child() {
		let mut app = setup();

		let projectile = app
			.world
			.spawn((
				_Projectile { ..default() },
				GlobalTransform::from_translation(Vec3::ZERO),
			))
			.id();

		app.update();
		app.update();

		let child_count = get_component_in_children::<Transform>(&projectile, &app).len();

		assert_eq!(1, child_count);
	}
}
