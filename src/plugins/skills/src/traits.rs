pub(crate) mod bevy_input;
pub(crate) mod force_shield;
pub(crate) mod inventory;
pub(crate) mod projectile;
pub(crate) mod skill_state;
pub(crate) mod state;
pub(crate) mod tuple_slot_key_item;

use crate::{
	components::slots::Slots,
	items::SlotKey,
	resources::SlotMap,
	skills::{
		Animate,
		Skill,
		SkillAnimation,
		SkillCaster,
		SkillExecution,
		SkillSpawner,
		StartBehaviorFn,
		StopBehaviorFn,
		Target,
	},
};
use animations::animation::Animation;
use bevy::ecs::{bundle::Bundle, system::EntityCommands};
use common::{
	tools::{Last, This},
	traits::{load_asset::Path, state_duration::StateUpdate},
};
use std::hash::Hash;

pub(crate) trait Enqueue<TItem> {
	fn enqueue(&mut self, item: TItem);
}

pub(crate) trait NewSkillBundle {
	type Bundle;
	fn new_bundle(caster: &SkillCaster, spawner: &SkillSpawner, target: &Target) -> Self::Bundle;
}

pub(crate) trait RunSkillAttached {
	fn run_attached(
		agent: &mut EntityCommands,
		caster: &SkillCaster,
		spawner: &SkillSpawner,
		target: &Target,
	);
}

impl<T: NewSkillBundle<Bundle = impl Bundle>> RunSkillAttached for T {
	fn run_attached(
		agent: &mut EntityCommands,
		caster: &SkillCaster,
		spawner: &SkillSpawner,
		target: &Target,
	) {
		agent.insert(T::new_bundle(caster, spawner, target));
	}
}

pub(crate) trait RunSkillDetached {
	fn run_detached(
		agent: &mut EntityCommands,
		caster: &SkillCaster,
		spawner: &SkillSpawner,
		target: &Target,
	);
}

impl<T: NewSkillBundle<Bundle = impl Bundle>> RunSkillDetached for T {
	fn run_detached(
		agent: &mut EntityCommands,
		caster: &SkillCaster,
		spawner: &SkillSpawner,
		target: &Target,
	) {
		let mut commands = agent.commands();
		commands.spawn(T::new_bundle(caster, spawner, target));
	}
}

pub(crate) trait StopSkillAttached {
	fn stop_attached(agent: &mut EntityCommands);
}

impl<T: NewSkillBundle<Bundle = TBundle>, TBundle: Bundle> StopSkillAttached for T {
	fn stop_attached(agent: &mut EntityCommands) {
		agent.remove::<TBundle>();
	}
}

pub(crate) trait Flush {
	fn flush(&mut self);
}

pub trait Iter<TItem> {
	fn iter<'a>(&'a self) -> impl DoubleEndedIterator<Item = &'a TItem>
	where
		TItem: 'a;
}

pub(crate) trait IterMutWithKeys<TKey, TItem> {
	fn iter_mut_with_keys<'a>(
		&'a mut self,
	) -> impl DoubleEndedIterator<Item = (TKey, &'a mut TItem)>
	where
		TItem: 'a;
}

pub(crate) trait IterAddedMut<TItem> {
	fn iter_added_mut<'a>(&'a mut self) -> impl DoubleEndedIterator<Item = &'a mut TItem>
	where
		TItem: 'a;
}

pub(crate) trait Prime {
	fn prime(&mut self);
}

pub(crate) trait GetActiveSkill<TAnimation, TSkillState: Clone> {
	fn get_active(
		&mut self,
	) -> Option<impl Execution + GetAnimation<TAnimation> + StateUpdate<TSkillState>>;
	fn clear_active(&mut self);
}

pub(crate) trait NextCombo {
	fn next(&mut self, trigger: &SlotKey, slots: &Slots) -> Option<Skill>;
}

pub(crate) trait GetAnimation<TAnimation> {
	fn animate(&self) -> Animate<TAnimation>;
}

pub trait GetExecution {
	fn execution() -> SkillExecution;
}

pub(crate) trait Execution {
	fn get_start(&self) -> Option<StartBehaviorFn>;
	fn get_stop(&self) -> Option<StopBehaviorFn>;
}

pub trait InputState<TKey: Eq + Hash> {
	fn just_pressed_slots(&self, map: &SlotMap<TKey>) -> Vec<SlotKey>;
	fn pressed_slots(&self, map: &SlotMap<TKey>) -> Vec<SlotKey>;
	fn just_released_slots(&self, map: &SlotMap<TKey>) -> Vec<SlotKey>;
}

pub trait ShouldEnqueue {
	fn should_enqueue(&self) -> bool;
}

pub trait SkillTemplate {
	fn skill() -> Skill;
}

#[derive(Clone)]
pub(crate) struct AnimationChainIf {
	pub this: fn() -> Path,
	pub last: fn() -> Path,
	pub then: fn() -> Path,
}

pub(crate) trait GetAnimationSetup {
	fn get_animation() -> SkillAnimation;
	fn get_chains() -> Vec<AnimationChainIf>;
}

pub(crate) trait GetSkillAnimation {
	fn animation() -> SkillAnimation;
}

fn apply_chain<T: GetAnimationSetup>(mut this: This<Animation>, last: Last<Animation>) {
	let chains = T::get_chains();
	let chain = chains
		.iter()
		.find(|chain| this.path == (chain.this)() && last.path == (chain.last)());

	let Some(chain) = chain else {
		return;
	};

	this.path = (chain.then)();
}

impl<T: GetAnimationSetup> GetSkillAnimation for T {
	fn animation() -> SkillAnimation {
		if T::get_chains().is_empty() {
			return T::get_animation();
		}

		let SkillAnimation {
			mut left,
			mut right,
		} = T::get_animation();

		left.update_fn = Some(apply_chain::<T>);
		right.update_fn = Some(apply_chain::<T>);
		SkillAnimation { left, right }
	}
}

#[cfg(test)]
mod test_animation_chain_skill_animation {
	use super::*;
	use animations::animation::PlayMode;
	use mockall::mock;

	macro_rules! mock_setup {
		($ident:ident) => {
			mock! {
				$ident {}
				impl GetAnimationSetup for $ident {
					fn get_animation() -> SkillAnimation;
					fn get_chains() -> Vec<AnimationChainIf>;
				}
			}
		};
	}

	mock_setup!(_MapAnimation);

	#[test]
	fn map_left_and_right_animation() {
		let left = Animation::new(Path::from("left"), PlayMode::Repeat);
		let right = Animation::new(Path::from("right"), PlayMode::Repeat);
		let get_animation = Mock_MapAnimation::get_animation_context();
		let get_chains = Mock_MapAnimation::get_chains_context();

		get_animation.expect().return_const(SkillAnimation {
			left: left.clone(),
			right: right.clone(),
		});
		get_chains.expect().return_const(vec![]);

		assert_eq!(
			SkillAnimation { left, right },
			Mock_MapAnimation::animation()
		)
	}

	mock_setup!(_CallChain);

	#[test]
	fn add_apply_chain_func_when_chains_present() {
		let mut left = Animation::new(Path::from("left"), PlayMode::Repeat);
		let mut right = Animation::new(Path::from("right"), PlayMode::Repeat);
		let get_animation = Mock_CallChain::get_animation_context();
		let get_chains = Mock_CallChain::get_chains_context();

		get_animation.expect().return_const(SkillAnimation {
			left: left.clone(),
			right: right.clone(),
		});
		get_chains.expect().return_const(vec![AnimationChainIf {
			last: || Path::from(""),
			this: || Path::from(""),
			then: || Path::from(""),
		}]);

		left.update_fn = Some(apply_chain::<Mock_CallChain>);
		right.update_fn = Some(apply_chain::<Mock_CallChain>);

		assert_eq!(SkillAnimation { left, right }, Mock_CallChain::animation())
	}

	mock_setup!(_ChainCombo);

	#[test]
	fn apply_chain_combo() {
		let get_chains = Mock_ChainCombo::get_chains_context();

		get_chains.expect().return_const(vec![AnimationChainIf {
			last: || Path::from("1"),
			this: || Path::from("2"),
			then: || Path::from("3"),
		}]);

		let mut this = Animation::new(Path::from("2"), PlayMode::Repeat);
		let last = Animation::new(Path::from("1"), PlayMode::Repeat);
		apply_chain::<Mock_ChainCombo>(This(&mut this), Last(&last));

		assert_eq!(Path::from("3"), this.path);
	}

	mock_setup!(_ThisMismatch);

	#[test]
	fn do_not_apply_chain_when_this_mismatch() {
		let get_chains = Mock_ThisMismatch::get_chains_context();

		get_chains.expect().return_const(vec![AnimationChainIf {
			last: || Path::from("1"),
			this: || Path::from("2"),
			then: || Path::from("3"),
		}]);

		let mut this = Animation::new(Path::from("2 mismatch"), PlayMode::Repeat);
		let last = Animation::new(Path::from("1"), PlayMode::Repeat);
		apply_chain::<Mock_ThisMismatch>(This(&mut this), Last(&last));

		assert_eq!(Path::from("2 mismatch"), this.path);
	}

	mock_setup!(_LastMismatch);

	#[test]
	fn do_not_apply_chain_when_last_mismatch() {
		let get_chains = Mock_LastMismatch::get_chains_context();

		get_chains.expect().return_const(vec![AnimationChainIf {
			last: || Path::from("1"),
			this: || Path::from("2"),
			then: || Path::from("3"),
		}]);

		let mut this = Animation::new(Path::from("2"), PlayMode::Repeat);
		let last = Animation::new(Path::from("1 mismatch"), PlayMode::Repeat);
		apply_chain::<Mock_LastMismatch>(This(&mut this), Last(&last));

		assert_eq!(Path::from("2"), this.path);
	}

	mock_setup!(_DifferentChain);

	#[test]
	fn apply_different_chain() {
		let get_chains = Mock_DifferentChain::get_chains_context();

		get_chains.expect().return_const(vec![AnimationChainIf {
			last: || Path::from("d1"),
			this: || Path::from("d2"),
			then: || Path::from("d3"),
		}]);

		let mut this = Animation::new(Path::from("d2"), PlayMode::Repeat);
		let last = Animation::new(Path::from("d1"), PlayMode::Repeat);
		apply_chain::<Mock_DifferentChain>(This(&mut this), Last(&last));

		assert_eq!(Path::from("d3"), this.path);
	}
}

#[cfg(test)]
mod test_run_skill_detached {
	use super::*;
	use crate::skills::SelectInfo;
	use bevy::{
		app::{App, Update},
		ecs::{
			component::Component,
			entity::Entity,
			system::{Commands, Query},
		},
		math::{Ray3d, Vec3},
		transform::components::{GlobalTransform, Transform},
	};
	use common::{
		components::Outdated,
		resources::ColliderInfo,
		test_tools::utils::SingleThreadedApp,
	};

	#[derive(Component, Debug, PartialEq)]
	struct _Skill {
		caster: SkillCaster,
		spawner: SkillSpawner,
		target: Target,
	}

	impl NewSkillBundle for _Skill {
		type Bundle = _Skill;

		fn new_bundle(
			caster: &SkillCaster,
			spawner: &SkillSpawner,
			target: &Target,
		) -> Self::Bundle {
			_Skill {
				caster: *caster,
				spawner: *spawner,
				target: target.clone(),
			}
		}
	}

	fn setup(caster: SkillCaster, spawner: SkillSpawner, target: Target) -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			move |mut commands: Commands, query: Query<Entity>| {
				for id in &query {
					let mut agent = commands.entity(id);
					_Skill::run_detached(&mut agent, &caster, &spawner, &target);
				}
			},
		);

		app
	}

	#[test]
	fn spawn_not_on_agent() {
		let entity = Entity::from_raw(42);
		let caster = SkillCaster(Transform::from_xyz(1., 2., 3.));
		let spawner = SkillSpawner(GlobalTransform::from_xyz(4., 5., 6.));
		let target = SelectInfo {
			ray: Ray3d::new(Vec3::ONE, Vec3::ONE),
			collision_info: Some(ColliderInfo {
				collider: Outdated {
					entity,
					component: GlobalTransform::from_xyz(7., 8., 9.),
				},
				root: None,
			}),
		};
		let mut app = setup(caster, spawner, target.clone());
		let agent = app.world.spawn_empty().id();

		app.update();

		let skill = app.world.iter_entities().find(|e| e.id() != agent);

		assert_eq!(
			Some(&_Skill {
				caster,
				spawner,
				target,
			}),
			skill.and_then(|s| s.get::<_Skill>())
		);
	}
}

#[cfg(test)]
mod test_run_skill_attached {
	use super::*;
	use crate::skills::SelectInfo;
	use bevy::{
		app::{App, Update},
		ecs::{
			component::Component,
			entity::Entity,
			system::{Commands, Query},
		},
		math::{Ray3d, Vec3},
		transform::components::{GlobalTransform, Transform},
	};
	use common::{
		components::Outdated,
		resources::ColliderInfo,
		test_tools::utils::SingleThreadedApp,
	};

	#[derive(Component, Debug, PartialEq)]
	struct _Skill {
		caster: SkillCaster,
		spawner: SkillSpawner,
		target: Target,
	}

	impl NewSkillBundle for _Skill {
		type Bundle = _Skill;

		fn new_bundle(
			caster: &SkillCaster,
			spawner: &SkillSpawner,
			target: &Target,
		) -> Self::Bundle {
			_Skill {
				caster: *caster,
				spawner: *spawner,
				target: target.clone(),
			}
		}
	}

	fn setup(caster: SkillCaster, spawner: SkillSpawner, target: Target) -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			move |mut commands: Commands, query: Query<Entity>| {
				for id in &query {
					let mut agent = commands.entity(id);
					_Skill::run_attached(&mut agent, &caster, &spawner, &target);
				}
			},
		);

		app
	}

	#[test]
	fn spawn_on_agent() {
		let entity = Entity::from_raw(42);
		let caster = SkillCaster(Transform::from_xyz(1., 2., 3.));
		let spawner = SkillSpawner(GlobalTransform::from_xyz(4., 5., 6.));
		let target = SelectInfo {
			ray: Ray3d::new(Vec3::ONE, Vec3::ONE),
			collision_info: Some(ColliderInfo {
				collider: Outdated {
					entity,
					component: GlobalTransform::from_xyz(7., 8., 9.),
				},
				root: None,
			}),
		};
		let mut app = setup(caster, spawner, target.clone());
		let agent = app.world.spawn_empty().id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&_Skill {
				caster,
				spawner,
				target,
			}),
			agent.get::<_Skill>()
		);
	}
}

#[cfg(test)]
mod test_stop_skill_attached {
	use super::*;
	use bevy::{
		app::{App, Update},
		ecs::{
			component::Component,
			entity::Entity,
			system::{Commands, Query},
		},
	};
	use common::test_tools::utils::SingleThreadedApp;

	#[derive(Component, Debug, PartialEq)]
	struct _Skill;

	impl NewSkillBundle for _Skill {
		type Bundle = _Skill;

		fn new_bundle(_: &SkillCaster, _: &SkillSpawner, _: &Target) -> Self::Bundle {
			todo!()
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			move |mut commands: Commands, query: Query<Entity>| {
				for id in &query {
					let mut agent = commands.entity(id);
					_Skill::stop_attached(&mut agent);
				}
			},
		);

		app
	}

	#[test]
	fn remove_from_agent() {
		let mut app = setup();
		let agent = app.world.spawn(_Skill).id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(None, agent.get::<_Skill>());
	}
}
