pub(crate) mod dto;
pub(crate) mod lifetime_definition;
pub(crate) mod shoot_hand_gun;

use crate::{
	behaviors::{
		build_skill_shape::{BuildSkillShape, OnSkillStop},
		spawn_on::SpawnOn,
		SkillBehaviorConfig,
		SkillCaster,
		SkillSpawner,
		Target,
	},
	item::item_type::SkillItemType,
	slot_key::SlotKey,
	traits::{spawn_skill_behavior::SpawnSkillBehavior, Matches, Prime},
};
use bevy::prelude::*;
use common::{
	effects::deal_damage::DealDamage,
	resources::ColliderInfo,
	traits::{
		animation::Animation,
		handles_effect::HandlesEffect,
		handles_lifetime::HandlesLifetime,
		load_asset::Path,
	},
};
use loading::traits::asset_folder::AssetFolderPath;
use std::{
	collections::HashSet,
	fmt::{Display, Formatter, Result},
	time::Duration,
};

#[derive(PartialEq, Debug, Clone)]
pub struct SkillAnimation {
	pub(crate) top_hand_left: Animation,
	pub(crate) top_hand_right: Animation,
	pub(crate) btm_hand_left: Animation,
	pub(crate) btm_hand_right: Animation,
}

#[derive(PartialEq, Debug, Default, Clone)]
pub enum Animate<TAnimation> {
	#[default]
	Ignore,
	None,
	Some(TAnimation),
}

#[derive(PartialEq, Debug, Default, Clone, TypePath, Asset)]
pub struct Skill {
	pub name: String,
	pub cast_time: Duration,
	pub animate: Animate<SkillAnimation>,
	pub behavior: RunSkillBehavior,
	pub is_usable_with: HashSet<SkillItemType>,
	pub icon: Option<Handle<Image>>,
}

impl Display for Skill {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		match self.name.as_str() {
			"" => write!(f, "Skill(<no name>)"),
			name => write!(f, "Skill({})", name),
		}
	}
}

impl AssetFolderPath for Skill {
	fn asset_folder_path() -> Path {
		Path::from("skills")
	}
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SelectInfo<T> {
	pub ray: Ray3d,
	pub collision_info: Option<ColliderInfo<T>>,
}

impl<T> Default for SelectInfo<T> {
	fn default() -> Self {
		Self {
			ray: Ray3d {
				origin: Vec3::ZERO,
				direction: Dir3::NEG_Z,
			},
			collision_info: None,
		}
	}
}

#[derive(Debug, PartialEq, Clone, Default)]
pub enum Activation {
	#[default]
	Waiting,
	Primed,
	ActiveAfter(Duration),
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct QueuedSkill {
	pub skill: Skill,
	pub slot_key: SlotKey,
	pub mode: Activation,
}

impl Prime for QueuedSkill {
	fn prime(&mut self) {
		if self.mode != Activation::Waiting {
			return;
		}
		self.mode = Activation::Primed;
	}
}

impl Matches<SlotKey> for QueuedSkill {
	fn matches(&self, slot_key: &SlotKey) -> bool {
		&self.slot_key == slot_key
	}
}

#[cfg(test)]
mod test_queued {
	use super::*;
	use bevy::utils::default;

	#[test]
	fn prime_skill() {
		let mut queued = QueuedSkill {
			skill: Skill::default(),
			mode: Activation::Waiting,
			..default()
		};
		queued.prime();

		assert_eq!(Activation::Primed, queued.mode);
	}

	#[test]
	fn do_not_prime_active() {
		let mut queued = QueuedSkill {
			skill: Skill::default(),
			mode: Activation::ActiveAfter(Duration::from_millis(123)),
			..default()
		};
		queued.prime();

		assert_eq!(
			Activation::ActiveAfter(Duration::from_millis(123)),
			queued.mode
		);
	}
}

#[derive(PartialEq, Debug, Clone, Copy, Eq, Hash)]
pub(crate) enum SkillState {
	Aim,
	Active,
}

#[derive(PartialEq, Debug, Clone)]
pub enum RunSkillBehavior {
	OnActive(SkillBehaviorConfig),
	OnAim(SkillBehaviorConfig),
}

impl Default for RunSkillBehavior {
	fn default() -> Self {
		Self::OnActive(SkillBehaviorConfig::from_shape(BuildSkillShape::NO_SHAPE))
	}
}

impl SpawnSkillBehavior<Commands<'_, '_>> for RunSkillBehavior {
	fn spawn_on(&self) -> SpawnOn {
		match self {
			RunSkillBehavior::OnActive(skill) => skill.spawn_on,
			RunSkillBehavior::OnAim(skill) => skill.spawn_on,
		}
	}

	fn spawn<TLifetimeDependency, TEffectDependency>(
		&self,
		commands: &mut Commands,
		caster: &SkillCaster,
		spawner: &SkillSpawner,
		target: &Target,
	) -> OnSkillStop
	where
		TLifetimeDependency: HandlesLifetime + 'static,
		TEffectDependency: HandlesEffect<DealDamage> + 'static,
	{
		match self {
			RunSkillBehavior::OnActive(conf) => spawn::<TLifetimeDependency, TEffectDependency>(
				conf, commands, caster, spawner, target,
			),
			RunSkillBehavior::OnAim(conf) => spawn::<TLifetimeDependency, TEffectDependency>(
				conf, commands, caster, spawner, target,
			),
		}
	}
}

fn spawn<TLifetimeDependency, TEffectDependency>(
	behavior: &SkillBehaviorConfig,
	commands: &mut Commands,
	caster: &SkillCaster,
	spawner: &SkillSpawner,
	target: &Target,
) -> OnSkillStop
where
	TLifetimeDependency: HandlesLifetime + 'static,
	TEffectDependency: HandlesEffect<DealDamage> + 'static,
{
	let shape = behavior.spawn_shape::<TLifetimeDependency>(commands, caster, spawner, target);

	if let Some(mut contact) = commands.get_entity(shape.contact) {
		behavior.start_contact_behavior::<TEffectDependency>(&mut contact, caster, spawner, target);
	};

	if let Some(mut projection) = commands.get_entity(shape.projection) {
		behavior.start_projection_behavior::<TEffectDependency>(
			&mut projection,
			caster,
			spawner,
			target,
		);
	};

	shape.on_skill_stop
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{behaviors::start_behavior::SkillBehavior, traits::skill_builder::SkillShape};
	use bevy::ecs::system::{EntityCommands, RunSystemOnce};
	use common::{components::Outdated, test_tools::utils::SingleThreadedApp};

	#[derive(Component, Debug, PartialEq)]
	struct _Args {
		caster: SkillCaster,
		spawner: SkillSpawner,
		target: Target,
	}

	#[derive(Component)]
	struct _Contact;

	#[derive(Component)]
	struct _Projection;

	struct _HandlesLifetime;

	impl HandlesLifetime for _HandlesLifetime {
		fn lifetime(_: Duration) -> impl Bundle {}
	}

	struct _HandlesEffects;

	impl<T> HandlesEffect<T> for _HandlesEffects
	where
		T: Sync + Send + 'static,
	{
		fn effect(_: T) -> impl Bundle {}
	}

	fn behavior(e: &mut EntityCommands, c: &SkillCaster, s: &SkillSpawner, t: &Target) {
		e.try_insert(_Args {
			caster: *c,
			spawner: *s,
			target: *t,
		});
	}

	fn get_target() -> Target {
		Target {
			ray: Ray3d::new(Vec3::new(1., 2., 3.), Vec3::new(4., 5., 6.)),
			collision_info: Some(ColliderInfo {
				collider: Outdated {
					entity: Entity::from_raw(11),
					component: GlobalTransform::from_xyz(10., 10., 10.),
				},
				root: Some(Outdated {
					entity: Entity::from_raw(1),
					component: GlobalTransform::from_xyz(11., 11., 11.),
				}),
			}),
		}
	}

	fn execute_callback<TCallback>(In(mut callback): In<TCallback>, mut cmd: Commands)
	where
		TCallback: FnMut(&mut Commands),
	{
		callback(&mut cmd);
	}

	fn spawned_args(app: &App, predicate: fn(&EntityRef) -> bool) -> Vec<&_Args> {
		app.world()
			.iter_entities()
			.filter(predicate)
			.filter_map(|e| e.get::<_Args>())
			.collect()
	}

	fn no_filter(_: &EntityRef) -> bool {
		true
	}

	fn filter<T: Component>(entity: &EntityRef) -> bool {
		entity.contains::<T>()
	}

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn spawn_skill_contact_entity_on_active() {
		let mut app = setup();
		let behavior = RunSkillBehavior::OnActive(
			SkillBehaviorConfig::from_shape(BuildSkillShape::Fn(|cmd, caster, spawner, target| {
				SkillShape {
					contact: cmd
						.spawn(_Args {
							caster: *caster,
							spawner: *spawner,
							target: *target,
						})
						.id(),
					projection: cmd.spawn_empty().id(),
					on_skill_stop: OnSkillStop::Ignore,
				}
			}))
			.spawning_on(SpawnOn::Slot),
		);
		let caster = SkillCaster(Entity::from_raw(1));
		let spawner = SkillSpawner(Entity::from_raw(2));
		let target = get_target();

		app.world_mut().run_system_once_with(
			move |cmd| {
				behavior
					.spawn::<_HandlesLifetime, _HandlesEffects>(cmd, &caster, &spawner, &target);
			},
			execute_callback,
		);

		assert_eq!(
			vec![&_Args {
				caster,
				spawner,
				target
			}],
			spawned_args(&app, no_filter)
		)
	}

	#[test]
	fn spawn_skill_contact_entity_on_active_centered() {
		let mut app = setup();
		let behavior = RunSkillBehavior::OnActive(
			SkillBehaviorConfig::from_shape(BuildSkillShape::Fn(|cmd, caster, spawner, target| {
				SkillShape {
					contact: cmd
						.spawn(_Args {
							caster: *caster,
							spawner: *spawner,
							target: *target,
						})
						.id(),
					projection: cmd.spawn_empty().id(),
					on_skill_stop: OnSkillStop::Ignore,
				}
			}))
			.spawning_on(SpawnOn::Center),
		);
		let caster = SkillCaster(Entity::from_raw(1));
		let spawner = SkillSpawner(Entity::from_raw(2));
		let target = get_target();

		app.world_mut().run_system_once_with(
			move |cmd| {
				behavior
					.spawn::<_HandlesLifetime, _HandlesEffects>(cmd, &caster, &spawner, &target);
			},
			execute_callback,
		);

		assert_eq!(
			vec![&_Args {
				caster,
				spawner,
				target
			}],
			spawned_args(&app, no_filter)
		);
	}

	#[test]
	fn apply_contact_behavior_on_active() {
		fn shape(cmd: &mut Commands, _: &SkillCaster, _: &SkillSpawner, _: &Target) -> SkillShape {
			SkillShape {
				contact: cmd.spawn(_Contact).id(),
				projection: cmd.spawn_empty().id(),
				on_skill_stop: OnSkillStop::Ignore,
			}
		}

		let mut app = setup();
		let behavior = RunSkillBehavior::OnActive(
			SkillBehaviorConfig::from_shape(BuildSkillShape::Fn(shape))
				.with_contact_behaviors(vec![SkillBehavior::Fn(behavior)]),
		);
		let caster = SkillCaster(Entity::from_raw(1));
		let spawner = SkillSpawner(Entity::from_raw(2));
		let target = get_target();

		app.world_mut().run_system_once_with(
			move |cmd| {
				behavior
					.spawn::<_HandlesLifetime, _HandlesEffects>(cmd, &caster, &spawner, &target);
			},
			execute_callback,
		);

		assert_eq!(
			vec![&_Args {
				caster,
				spawner,
				target
			}],
			spawned_args(&app, filter::<_Contact>)
		);
	}

	#[test]
	fn spawn_skill_projection_entity_on_active() {
		let mut app = setup();
		let behavior = RunSkillBehavior::OnActive(SkillBehaviorConfig::from_shape(
			BuildSkillShape::Fn(|cmd, caster, spawner, target| SkillShape {
				contact: cmd
					.spawn(_Args {
						caster: *caster,
						spawner: *spawner,
						target: *target,
					})
					.id(),
				projection: cmd.spawn_empty().id(),
				on_skill_stop: OnSkillStop::Ignore,
			}),
		));
		let caster = SkillCaster(Entity::from_raw(1));
		let spawner = SkillSpawner(Entity::from_raw(2));
		let target = get_target();

		app.world_mut().run_system_once_with(
			move |cmd| {
				behavior
					.spawn::<_HandlesLifetime, _HandlesEffects>(cmd, &caster, &spawner, &target);
			},
			execute_callback,
		);

		assert_eq!(
			vec![&_Args {
				caster,
				spawner,
				target
			}],
			spawned_args(&app, no_filter)
		);
	}

	#[test]
	fn apply_projection_behavior_on_active() {
		fn shape(cmd: &mut Commands, _: &SkillCaster, _: &SkillSpawner, _: &Target) -> SkillShape {
			SkillShape {
				contact: cmd.spawn_empty().id(),
				projection: cmd.spawn(_Projection).id(),
				on_skill_stop: OnSkillStop::Ignore,
			}
		}

		let mut app = setup();
		let behavior = RunSkillBehavior::OnActive(
			SkillBehaviorConfig::from_shape(BuildSkillShape::Fn(shape))
				.with_projection_behaviors(vec![SkillBehavior::Fn(behavior)]),
		);
		let caster = SkillCaster(Entity::from_raw(1));
		let spawner = SkillSpawner(Entity::from_raw(2));
		let target = get_target();

		app.world_mut().run_system_once_with(
			move |cmd| {
				behavior
					.spawn::<_HandlesLifetime, _HandlesEffects>(cmd, &caster, &spawner, &target);
			},
			execute_callback,
		);

		let spawn_args = app
			.world()
			.iter_entities()
			.filter(|e| e.contains::<_Projection>())
			.filter_map(|e| e.get::<_Args>())
			.collect::<Vec<_>>();

		assert_eq!(
			vec![&_Args {
				caster,
				spawner,
				target
			}],
			spawn_args
		);
	}

	#[test]
	fn spawn_skill_contact_entity_on_aim() {
		let mut app = setup();
		let behavior = RunSkillBehavior::OnAim(
			SkillBehaviorConfig::from_shape(BuildSkillShape::Fn(|cmd, caster, spawner, target| {
				SkillShape {
					contact: cmd
						.spawn(_Args {
							caster: *caster,
							spawner: *spawner,
							target: *target,
						})
						.id(),
					projection: cmd.spawn_empty().id(),
					on_skill_stop: OnSkillStop::Ignore,
				}
			}))
			.spawning_on(SpawnOn::Slot),
		);
		let caster = SkillCaster(Entity::from_raw(1));
		let spawner = SkillSpawner(Entity::from_raw(2));
		let target = get_target();

		app.world_mut().run_system_once_with(
			move |cmd| {
				behavior
					.spawn::<_HandlesLifetime, _HandlesEffects>(cmd, &caster, &spawner, &target);
			},
			execute_callback,
		);

		assert_eq!(
			vec![&_Args {
				caster,
				spawner,
				target
			}],
			spawned_args(&app, no_filter)
		);
	}

	#[test]
	fn spawn_skill_contact_entity_on_aim_centered() {
		let mut app = setup();
		let behavior = RunSkillBehavior::OnAim(
			SkillBehaviorConfig::from_shape(BuildSkillShape::Fn(|cmd, caster, spawner, target| {
				SkillShape {
					contact: cmd
						.spawn(_Args {
							caster: *caster,
							spawner: *spawner,
							target: *target,
						})
						.id(),
					projection: cmd.spawn_empty().id(),
					on_skill_stop: OnSkillStop::Ignore,
				}
			}))
			.spawning_on(SpawnOn::Center),
		);
		let caster = SkillCaster(Entity::from_raw(1));
		let spawner = SkillSpawner(Entity::from_raw(2));
		let target = get_target();

		app.world_mut().run_system_once_with(
			move |cmd| {
				behavior
					.spawn::<_HandlesLifetime, _HandlesEffects>(cmd, &caster, &spawner, &target);
			},
			execute_callback,
		);

		assert_eq!(
			vec![&_Args {
				caster,
				spawner,
				target
			}],
			spawned_args(&app, no_filter)
		);
	}

	#[test]
	fn apply_contact_behavior_on_aim() {
		#[derive(Component)]
		struct _Contact;

		fn shape(cmd: &mut Commands, _: &SkillCaster, _: &SkillSpawner, _: &Target) -> SkillShape {
			SkillShape {
				contact: cmd.spawn(_Contact).id(),
				projection: cmd.spawn_empty().id(),
				on_skill_stop: OnSkillStop::Ignore,
			}
		}

		let mut app = setup();
		let behavior = RunSkillBehavior::OnAim(
			SkillBehaviorConfig::from_shape(BuildSkillShape::Fn(shape))
				.with_contact_behaviors(vec![SkillBehavior::Fn(behavior)]),
		);
		let caster = SkillCaster(Entity::from_raw(1));
		let spawner = SkillSpawner(Entity::from_raw(2));
		let target = get_target();

		app.world_mut().run_system_once_with(
			move |cmd| {
				behavior
					.spawn::<_HandlesLifetime, _HandlesEffects>(cmd, &caster, &spawner, &target);
			},
			execute_callback,
		);

		let spawn_args = app
			.world()
			.iter_entities()
			.filter(|e| e.contains::<_Contact>())
			.filter_map(|e| e.get::<_Args>())
			.collect::<Vec<_>>();

		assert_eq!(
			vec![&_Args {
				caster,
				spawner,
				target
			}],
			spawn_args
		);
	}

	#[test]
	fn spawn_skill_projection_entity_on_aim() {
		let mut app = setup();
		let behavior = RunSkillBehavior::OnAim(SkillBehaviorConfig::from_shape(
			BuildSkillShape::Fn(|cmd, caster, spawner, target| SkillShape {
				contact: cmd
					.spawn(_Args {
						caster: *caster,
						spawner: *spawner,
						target: *target,
					})
					.id(),
				projection: cmd.spawn_empty().id(),
				on_skill_stop: OnSkillStop::Ignore,
			}),
		));
		let caster = SkillCaster(Entity::from_raw(1));
		let spawner = SkillSpawner(Entity::from_raw(2));
		let target = get_target();

		app.world_mut().run_system_once_with(
			move |cmd| {
				behavior
					.spawn::<_HandlesLifetime, _HandlesEffects>(cmd, &caster, &spawner, &target);
			},
			execute_callback,
		);

		assert_eq!(
			vec![&_Args {
				caster,
				spawner,
				target
			}],
			spawned_args(&app, no_filter)
		);
	}

	#[test]
	fn apply_projection_behavior_on_aim() {
		#[derive(Component)]
		struct _Projection;

		fn shape(cmd: &mut Commands, _: &SkillCaster, _: &SkillSpawner, _: &Target) -> SkillShape {
			SkillShape {
				contact: cmd.spawn_empty().id(),
				projection: cmd.spawn(_Projection).id(),
				on_skill_stop: OnSkillStop::Ignore,
			}
		}

		let mut app = setup();
		let behavior = RunSkillBehavior::OnAim(
			SkillBehaviorConfig::from_shape(BuildSkillShape::Fn(shape))
				.with_projection_behaviors(vec![SkillBehavior::Fn(behavior)]),
		);
		let caster = SkillCaster(Entity::from_raw(1));
		let spawner = SkillSpawner(Entity::from_raw(2));
		let target = get_target();

		app.world_mut().run_system_once_with(
			move |cmd| {
				behavior
					.spawn::<_HandlesLifetime, _HandlesEffects>(cmd, &caster, &spawner, &target);
			},
			execute_callback,
		);

		let spawn_args = app
			.world()
			.iter_entities()
			.filter(|e| e.contains::<_Projection>())
			.filter_map(|e| e.get::<_Args>())
			.collect::<Vec<_>>();

		assert_eq!(
			vec![&_Args {
				caster,
				spawner,
				target
			}],
			spawn_args
		);
	}
}
