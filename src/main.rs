use bevy::prelude::*;

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)
		.add_systems(Startup, setup_simple_3d_scene)
		.run();
}

fn setup_simple_3d_scene(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	spawn_plane(&mut commands, &mut meshes, &mut materials);
	spawn_cube(&mut commands, meshes, materials);
	spawn_light(&mut commands);
	spawn_camera(&mut commands);
}

fn spawn_plane(
	commands: &mut Commands,
	meshes: &mut ResMut<Assets<Mesh>>,
	materials: &mut ResMut<Assets<StandardMaterial>>,
) {
	commands.spawn(PbrBundle {
		mesh: meshes.add(shape::Plane::from_size(5.0).into()),
		material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
		..default()
	});
}

fn spawn_cube(
	commands: &mut Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	commands.spawn(PbrBundle {
		mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
		material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
		transform: Transform::from_xyz(0.0, 0.5, 0.0),
		..default()
	});
}

fn spawn_light(commands: &mut Commands) {
	commands.spawn(PointLightBundle {
		point_light: PointLight {
			intensity: 1500.0,
			shadows_enabled: true,
			..default()
		},
		transform: Transform::from_xyz(4.0, 8.0, 4.0),
		..default()
	});
}

fn spawn_camera(commands: &mut Commands) {
	commands.spawn(Camera3dBundle {
		transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
		..default()
	});
}
