use behaviors::components::MovementConfig;
use bevy::ecs::{
	component::Component,
	entity::Entity,
	query::With,
	removal_detection::RemovedComponents,
	system::{Commands, Query},
};
use common::components::Animate;

type WithMovingAgent<TAgent, TMovement> = (With<TAgent>, With<TMovement>);

pub(crate) fn set_movement_animation<
	TAgent: Component,
	TMovement: Component,
	TAnimationKey: From<MovementConfig> + Copy + Send + Sync + 'static,
>(
	mut commands: Commands,
	moving_agents: Query<(Entity, &MovementConfig), WithMovingAgent<TAgent, TMovement>>,
	mut stopped_agents: RemovedComponents<TMovement>,
) {
	for (id, config) in &moving_agents {
		commands
			.entity(id)
			.insert(Animate::Repeat(TAnimationKey::from(*config)));
	}
	for id in stopped_agents.read() {
		if let Some(mut entity) = commands.get_entity(id) {
			entity.remove::<Animate<TAnimationKey>>();
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use behaviors::components::MovementMode;
	use bevy::{
		app::{App, Update},
		utils::default,
	};

	#[derive(Clone, Copy, PartialEq, Debug)]
	enum _Key {
		Constant,
		Dynamic,
	}

	impl From<MovementConfig> for _Key {
		fn from(config: MovementConfig) -> Self {
			match config {
				MovementConfig::Constant { .. } => _Key::Constant,
				MovementConfig::Dynamic { .. } => _Key::Dynamic,
			}
		}
	}

	#[derive(Component)]
	struct _Agent;

	#[derive(Component)]
	struct _Movement;

	fn setup() -> App {
		let mut app = App::new();
		app.add_systems(Update, set_movement_animation::<_Agent, _Movement, _Key>);

		app
	}

	#[test]
	fn add_animate() {
		let mut app = setup();
		let agent = app
			.world
			.spawn((
				_Agent,
				_Movement,
				MovementConfig::Constant {
					mode: MovementMode::Fast,
					speed: default(),
				},
			))
			.id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&Animate::Repeat(_Key::Constant)),
			agent.get::<Animate<_Key>>()
		);
	}

	#[test]
	fn do_not_add_animate_if_no_agent_present() {
		let mut app = setup();
		let agent = app
			.world
			.spawn((
				_Movement,
				MovementConfig::Constant {
					mode: MovementMode::Fast,
					speed: default(),
				},
			))
			.id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(None, agent.get::<Animate<_Key>>());
	}

	#[test]
	fn do_not_add_animate_if_no_movement_present() {
		let mut app = setup();
		let agent = app
			.world
			.spawn((
				_Agent,
				MovementConfig::Constant {
					mode: MovementMode::Fast,
					speed: default(),
				},
			))
			.id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(None, agent.get::<Animate<_Key>>());
	}

	#[test]
	fn remove_animate_when_movement_removed() {
		let mut app = setup();
		let agent = app
			.world
			.spawn((
				_Agent,
				_Movement,
				MovementConfig::Constant {
					mode: MovementMode::Fast,
					speed: default(),
				},
			))
			.id();

		app.update();

		app.world.entity_mut(agent).remove::<_Movement>();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(None, agent.get::<Animate<_Key>>());
	}
}
