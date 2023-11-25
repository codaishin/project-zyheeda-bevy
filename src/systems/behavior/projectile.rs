use crate::{
	components::{Projectile, SimpleMovement, WaitNext},
	errors::Error,
	resources::Models,
};
use bevy::{
	ecs::{
		query::{Added, With},
		system::{Commands, Query},
	},
	hierarchy::{BuildChildren, DespawnRecursiveExt},
	math::Vec3,
	prelude::{default, Entity, Res},
	scene::SceneBundle,
	transform::components::GlobalTransform,
};

const KEY_ERROR: &str = "'projectile' model not found";

pub fn projectile(
	mut commands: Commands,
	active_agents: Query<(Entity, &Projectile, &GlobalTransform), Added<Projectile>>,
	inactive_agents: Query<Entity, (With<WaitNext>, With<Projectile>)>,
	models: Res<Models>,
) -> Result<(), Error> {
	for entity in &inactive_agents {
		commands.entity(entity).despawn_recursive();
	}

	if active_agents.is_empty() {
		return Ok(());
	}

	let Some(scene) = models.0.get("projectile") else {
		return Err(Error(KEY_ERROR.to_owned()));
	};

	for (entity, projectile, transform) in &active_agents {
		if let Some(target) = get_target(transform, projectile) {
			let model = commands
				.spawn(SceneBundle {
					scene: scene.clone(),
					..default()
				})
				.id();
			commands
				.entity(entity)
				.insert(SimpleMovement { target })
				.add_child(model);
		}
	}

	Ok(())
}

fn get_target(transform: &GlobalTransform, projectile: &Projectile) -> Option<Vec3> {
	let current_position = transform.translation();
	let ray = projectile.target_ray;
	let target = ray.origin + ray.direction * ray.intersect_plane(current_position, Vec3::Y)?;
	let direction = (target - current_position).normalize();

	Some(current_position + direction * projectile.range)
}

#[cfg(test)]
mod tests {
	use crate::components::{SimpleMovement, WaitNext};

	use super::*;
	use bevy::{
		app::{App, Update},
		asset::{AssetId, Handle},
		ecs::{
			component::Component,
			system::{In, IntoSystem},
		},
		hierarchy::{BuildWorldChildren, Children},
		math::{Ray, Vec3},
		scene::Scene,
		transform::components::GlobalTransform,
		utils::Uuid,
	};

	#[derive(Component)]
	struct MockLog(pub Result<(), Error>);

	type LoggerEntity = Entity;

	const SIMPLE_RAY: Ray = Ray {
		origin: Vec3::Y,
		direction: Vec3::NEG_Y,
	};

	fn log_result(result: In<Result<(), Error>>, mut loggers: Query<&mut MockLog>) {
		let mut logger = loggers.single_mut();
		logger.0 = result.0;
	}

	fn setup<const N: usize>(
		model_data: [(&'static str, Handle<Scene>); N],
	) -> (App, LoggerEntity) {
		let mut app = App::new();
		let logger = app
			.world
			.spawn(MockLog(Err(Error("Initial Fake Error".to_owned()))))
			.id();
		app.add_systems(Update, projectile.pipe(log_result));
		app.insert_resource(Models(model_data.into()));

		(app, logger)
	}

	#[test]
	fn spawn_model() {
		let model = Handle::<Scene>::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let (mut app, logger) = setup([("projectile", model.clone())]);
		let projectile = app
			.world
			.spawn((
				Projectile {
					target_ray: SIMPLE_RAY,
					range: 1.,
				},
				GlobalTransform::from_xyz(0., 0., 0.),
			))
			.id();

		app.update();

		let logger = app.world.entity(logger);
		let projectile = app.world.entity(projectile);
		let system_return = logger.get::<MockLog>().unwrap().0.clone();
		let projectile_model_on_child = projectile.get::<Children>().and_then(|children| {
			children
				.iter()
				.map(|child| app.world.entity(*child))
				.find_map(|child| child.get::<Handle<Scene>>())
		});

		assert_eq!(
			(Ok(()), Some(&model)),
			(system_return, projectile_model_on_child)
		);
	}

	#[test]
	fn log_error_when_no_model_available() {
		let (mut app, logger) = setup([]);
		let projectile = app
			.world
			.spawn((
				Projectile {
					target_ray: SIMPLE_RAY,
					range: 1.,
				},
				GlobalTransform::from_xyz(0., 0., 0.),
			))
			.id();

		app.update();

		let logger = app.world.entity(logger);
		let projectile = app.world.entity(projectile);
		let system_return = logger.get::<MockLog>().unwrap().0.clone();
		let projectile_model = projectile.get::<Handle<Scene>>();

		assert_eq!(
			(Err(Error(KEY_ERROR.to_owned())), None),
			(system_return, projectile_model)
		);
	}

	#[test]
	fn do_not_log_when_not_projectile_exists() {
		let (mut app, logger) = setup([]);

		app.update();

		let logger = app.world.entity(logger);
		let result = logger.get::<MockLog>().unwrap().0.clone();

		assert_eq!(Ok(()), result);
	}

	#[test]
	fn insert_simple_movement() {
		let (mut app, ..) = setup([("projectile", Handle::default())]);
		let projectile = app
			.world
			.spawn((
				Projectile {
					target_ray: Ray {
						origin: Vec3::ONE,
						direction: Vec3::NEG_Y,
					},
					range: 1.,
				},
				GlobalTransform::from_xyz(0., 0., 0.),
			))
			.id();

		app.update();

		let projectile = app.world.entity(projectile);

		assert!(projectile.contains::<SimpleMovement>());
	}

	#[test]
	fn compute_target_on_current_spawn_height_and_range() {
		let (mut app, ..) = setup([("projectile", Handle::default())]);
		let projectile = app
			.world
			.spawn((
				Projectile {
					target_ray: Ray {
						origin: Vec3::new(7., 20., 1.),
						direction: Vec3::NEG_Y,
					},
					range: 5.,
				},
				GlobalTransform::from_xyz(4., 10., 1.),
			))
			.id();

		app.update();

		let projectile = app.world.entity(projectile);
		let simple_movement = projectile.get::<SimpleMovement>().unwrap();

		assert_eq!(
			&SimpleMovement {
				target: Vec3::new(9., 10., 1.)
			},
			simple_movement
		);
	}

	#[test]
	fn despawn_when_waiting_next() {
		let (mut app, ..) = setup([]);

		let child = app.world.spawn(()).id();
		let projectile = app
			.world
			.spawn((
				Projectile {
					target_ray: Ray::default(),
					range: 1.,
				},
				GlobalTransform::from_xyz(0., 0., 0.),
			))
			.add_child(child)
			.id();

		app.update();

		app.world.entity_mut(projectile).insert(WaitNext);

		app.update();

		let projectile = app.world.get_entity(projectile);
		let child = app.world.get_entity(child);

		assert_eq!((true, true), (projectile.is_none(), child.is_none()));
	}

	#[test]
	fn do_not_despawn_non_projectiles() {
		let (mut app, ..) = setup([]);
		let non_projectile = app.world.spawn(WaitNext).id();

		app.update();

		let non_projectile = app.world.get_entity(non_projectile);

		assert!(non_projectile.is_some());
	}

	#[test]
	fn do_nothing_when_no_valid_target() {
		let (mut app, ..) = setup([("projectile", Handle::default())]);
		let projectile = app
			.world
			.spawn((
				Projectile {
					target_ray: Ray::default(),
					range: 1.,
				},
				GlobalTransform::from_xyz(0., 0., 0.),
			))
			.id();

		app.update();

		let projectile = app.world.entity(projectile);

		assert_eq!(
			(false, false),
			(
				projectile.contains::<SimpleMovement>(),
				projectile.contains::<Handle<Scene>>()
			)
		);
	}
}
