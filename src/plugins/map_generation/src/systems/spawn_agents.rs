use crate::resources::agents::prefab::AgentPrefab;
use bevy::prelude::*;
use common::{
	traits::handles_map_generation::AgentType,
	zyheeda_commands::{ZyheedaCommands, ZyheedaEntityCommands},
};
use std::collections::HashMap;

impl AgentPrefab {
	pub(crate) fn spawn_agents<const N: usize>(
		spawn_config: [(&'static str, AgentType); N],
	) -> impl IntoSystem<(), (), ()> {
		let spawn_config = HashMap::from(spawn_config);

		IntoSystem::into_system(
			move |agent_prefab: Res<Self>,
			      mut commands: ZyheedaCommands,
			      spawns: Query<(&Name, &GlobalTransform), Added<Name>>| {
				for (name, transform) in spawns {
					let Some(name) = get_base_name(name) else {
						continue;
					};
					let Some(agent_type) = spawn_config.get(name) else {
						continue;
					};

					let entity = ZyheedaEntityCommands::from(commands.spawn(*transform));
					(agent_prefab.0)(entity, transform.translation(), *agent_type);
				}
			},
		)
	}
}

fn get_base_name(name: &Name) -> Option<&str> {
	let Some(dot) = name.find(".") else {
		return Some(name.as_str());
	};

	if name.chars().skip(dot + 1).any(|c| !c.is_ascii_digit()) {
		return None;
	}

	Some(&name[0..dot])
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::traits::handles_enemies::EnemyType;
	use testing::{SingleThreadedApp, assert_count};

	#[derive(Component, Debug, PartialEq)]
	struct _Agent {
		ground_position: Vec3,
		agent_type: AgentType,
	}

	fn setup<const N: usize>(spawns: [(&'static str, AgentType); N]) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(AgentPrefab(|mut e, ground_position, agent_type| {
			e.try_insert(_Agent {
				ground_position,
				agent_type,
			});
		}));
		app.add_systems(Update, AgentPrefab::spawn_agents(spawns));

		app
	}

	#[test]
	fn spawn_agent() {
		let mut app = setup([
			("AA", AgentType::Player),
			("BB", AgentType::Enemy(EnemyType::VoidSphere)),
		]);
		app.world_mut()
			.spawn((Name::from("AA"), GlobalTransform::from_xyz(1., 2., 3.)));
		app.world_mut()
			.spawn((Name::from("BB"), GlobalTransform::from_xyz(4., 5., 6.)));

		app.update();

		let mut agents = app.world_mut().query::<&_Agent>();
		let agents = assert_count!(2, agents.iter(app.world()));
		assert_eq!(
			[
				&_Agent {
					ground_position: Vec3::new(1., 2., 3.),
					agent_type: AgentType::Player
				},
				&_Agent {
					ground_position: Vec3::new(4., 5., 6.),
					agent_type: AgentType::Enemy(EnemyType::VoidSphere)
				},
			],
			agents
		);
	}

	#[test]
	fn match_agent_to_name_until_dot() {
		let mut app = setup([
			("AA", AgentType::Player),
			("BB", AgentType::Enemy(EnemyType::VoidSphere)),
		]);
		app.world_mut()
			.spawn((Name::from("AA.11"), GlobalTransform::from_xyz(1., 2., 3.)));
		app.world_mut()
			.spawn((Name::from("BB.42"), GlobalTransform::from_xyz(4., 5., 6.)));

		app.update();

		let mut agents = app.world_mut().query::<&_Agent>();
		let agents = assert_count!(2, agents.iter(app.world()));
		assert_eq!(
			[
				&_Agent {
					ground_position: Vec3::new(1., 2., 3.),
					agent_type: AgentType::Player
				},
				&_Agent {
					ground_position: Vec3::new(4., 5., 6.),
					agent_type: AgentType::Enemy(EnemyType::VoidSphere)
				},
			],
			agents
		);
	}

	#[test]
	fn ignore_if_after_dot_not_numerical() {
		let mut app = setup([
			("AA", AgentType::Player),
			("BB", AgentType::Enemy(EnemyType::VoidSphere)),
		]);
		app.world_mut().spawn((
			Name::from("AA.material"),
			GlobalTransform::from_xyz(1., 2., 3.),
		));

		app.update();

		let mut agents = app.world_mut().query::<&_Agent>();
		assert_count!(0, agents.iter(app.world()));
	}

	#[test]
	fn apply_translation() {
		let mut app = setup([
			("AA", AgentType::Player),
			("BB", AgentType::Enemy(EnemyType::VoidSphere)),
		]);
		app.world_mut().spawn((
			Name::from("AA"),
			GlobalTransform::from(Transform::from_xyz(1., 2., 3.).looking_to(Dir3::X, Dir3::Y)),
		));
		app.world_mut().spawn((
			Name::from("BB"),
			GlobalTransform::from(Transform::from_xyz(4., 5., 6.).looking_to(Dir3::Z, Dir3::Y)),
		));

		app.update();

		let mut agents = app
			.world_mut()
			.query_filtered::<&GlobalTransform, Without<Name>>();
		let agents = assert_count!(2, agents.iter(app.world()));
		assert_eq!(
			[
				&GlobalTransform::from(
					Transform::from_xyz(1., 2., 3.).looking_to(Dir3::X, Dir3::Y)
				),
				&GlobalTransform::from(
					Transform::from_xyz(4., 5., 6.).looking_to(Dir3::Z, Dir3::Y)
				),
			],
			agents,
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup([("AA", AgentType::Player)]);
		app.world_mut()
			.spawn((Name::from("AA"), GlobalTransform::default()));

		app.update();
		app.update();

		let mut agents = app.world_mut().query::<&_Agent>();
		assert_count!(1, agents.iter(app.world()));
	}
}
