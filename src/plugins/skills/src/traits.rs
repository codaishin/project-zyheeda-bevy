pub(crate) mod bevy_input;
pub(crate) mod game_state;
pub(crate) mod inventory;
pub(crate) mod mouse_hover;
pub(crate) mod player_animation_states;
pub(crate) mod projectile;
pub(crate) mod queue;
pub(crate) mod skill;
pub(crate) mod skill_combo_next;
pub(crate) mod skill_state;
pub(crate) mod state;
pub(crate) mod sword;
pub(crate) mod track;
pub(crate) mod tuple_slot_key_item;

use crate::{
	components::SlotKey,
	resources::SlotMap,
	skill::{Active, Skill, SkillComboTree, SkillExecution, Spawner},
};
use bevy::{
	ecs::{
		component::Component,
		system::{EntityCommands, Query},
	},
	transform::components::Transform,
};
use common::{
	components::{Animate, Outdated},
	resources::ColliderInfo,
};
use std::hash::Hash;

pub(crate) trait ComboNext<TAnimationKey>
where
	Self: Sized,
{
	fn to_vec(&self, skill: &Skill<TAnimationKey, Active>) -> Vec<(SlotKey, SkillComboTree<Self>)>;
}

pub(crate) trait GetAnimation<TAnimationKey: Clone + Copy> {
	fn animate(&self) -> Animate<TAnimationKey>;
}

pub(crate) trait HasIdle<TAnimationKey: Clone + Copy> {
	const IDLE: Animate<TAnimationKey>;
}

pub(crate) trait WithComponent<T: Component + Copy> {
	fn with_component(&self, query: &Query<&T>) -> Option<ColliderInfo<Outdated<T>>>;
}

pub trait GetExecution {
	fn execution() -> SkillExecution;
}

pub(crate) trait Execution {
	fn run(&self, agent: &mut EntityCommands, agent_transform: &Transform, spawner: &Spawner);
	fn stop(&self, agent: &mut EntityCommands);
}

pub trait InputState<TKey: Eq + Hash> {
	fn just_pressed_slots(&self, map: &SlotMap<TKey>) -> Vec<SlotKey>;
	fn pressed_slots(&self, map: &SlotMap<TKey>) -> Vec<SlotKey>;
	fn just_released_slots(&self, map: &SlotMap<TKey>) -> Vec<SlotKey>;
}

pub trait ShouldEnqueue {
	fn should_enqueue(&self) -> bool;
}

#[cfg(test)]
pub(crate) mod test_tools {
	use super::*;
	use crate::skill::{Spawner, Target};
	use bevy::{
		ecs::{
			component::Component,
			system::{Commands, Query},
		},
		prelude::Entity,
		transform::components::Transform,
	};

	pub(crate) fn run_system<TExecute: Execution + Component>(
		agent: Entity,
		agent_transform: Transform,
		spawner: Spawner,
	) -> impl FnMut(Commands, Query<&mut TExecute>) {
		move |mut commands, mut executes| {
			let execute = executes.single_mut();
			execute.run(&mut commands.entity(agent), &agent_transform, &spawner);
		}
	}

	pub(crate) fn stop_system<TExecute: Execution + Component>(
		agent: Entity,
	) -> impl FnMut(Commands, Query<&TExecute>) {
		move |mut commands, executes| {
			let execute = executes.single();
			execute.stop(&mut commands.entity(agent));
		}
	}

	pub fn run_lazy(
		behavior: SkillExecution,
		agent: Entity,
		agent_transform: Transform,
		spawner: Spawner,
		select_info: Target,
	) -> impl FnMut(Commands) {
		move |mut commands| {
			let Some(run) = behavior.run_fn else {
				return;
			};
			let mut agent = commands.entity(agent);
			run(&mut agent, &agent_transform, &spawner, &select_info);
		}
	}
}
