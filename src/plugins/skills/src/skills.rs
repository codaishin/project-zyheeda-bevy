pub(crate) mod dto;
pub(crate) mod lifetime_definition;

use crate::{
	behaviors::{
		SkillBehaviorConfig,
		SkillCaster,
		spawn_skill::{OnSkillStop, SpawnOn},
	},
	components::SkillTarget,
	traits::{ReleaseSkill, spawn_skill_behavior::SpawnSkillBehavior},
};
use bevy::prelude::*;
use common::{
	tools::{
		action_key::slot::SlotKey,
		item_type::{CompatibleItems, ItemType},
	},
	traits::{
		accessors::get::{GetMut, GetProperty},
		handles_custom_assets::AssetFolderPath,
		handles_loadout::loadout::{SkillIcon, SkillToken},
		handles_localization::Token,
		handles_physics::HandlesAllPhysicalEffects,
		handles_skill_behaviors::{HandlesSkillBehaviors, SkillSpawner},
		load_asset::Path,
	},
	zyheeda_commands::ZyheedaCommands,
};
use serde::{Deserialize, Serialize};
use std::{
	collections::HashSet,
	fmt::{Display, Formatter, Result as FmtResult},
	time::Duration,
};

#[derive(PartialEq, Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub enum AnimationStrategy {
	#[default]
	None,
	DoNotAnimate,
	Animate,
}

#[derive(PartialEq, Debug, Clone, TypePath, Asset)]
#[cfg_attr(test, derive(Default))]
pub struct Skill {
	pub token: Token,
	pub cast_time: Duration,
	pub animation: AnimationStrategy,
	pub behavior: RunSkillBehavior,
	pub compatible_items: CompatibleItems,
	pub icon: Handle<Image>,
}

impl Display for Skill {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match &*self.token {
			"" => write!(f, "Skill(<no token>)"),
			name => write!(f, "Skill({name})"),
		}
	}
}

impl AssetFolderPath for Skill {
	fn asset_folder_path() -> Path {
		Path::from("skills")
	}
}

impl GetProperty<SkillToken> for Skill {
	fn get_property(&self) -> &Token {
		&self.token
	}
}

impl GetProperty<SkillIcon> for Skill {
	fn get_property(&self) -> &Handle<Image> {
		&self.icon
	}
}

impl GetProperty<CompatibleItems> for Skill {
	fn get_property(&self) -> &HashSet<ItemType> {
		&self.compatible_items.0
	}
}

#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
pub enum SkillMode {
	#[default]
	Hold,
	Release,
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(test, derive(Default))]
pub struct QueuedSkill {
	pub skill: Skill,
	pub key: SlotKey,
	pub skill_mode: SkillMode,
}

impl QueuedSkill {
	pub(crate) fn new(skill: Skill, key: SlotKey) -> Self {
		Self {
			skill,
			key,
			skill_mode: SkillMode::Hold,
		}
	}
}

impl GetProperty<SlotKey> for QueuedSkill {
	fn get_property(&self) -> SlotKey {
		self.key
	}
}

impl GetProperty<Token> for QueuedSkill {
	fn get_property(&self) -> &Token {
		&self.skill.token
	}
}

impl ReleaseSkill for QueuedSkill {
	fn release_skill(&mut self) {
		self.skill_mode = SkillMode::Release;
	}
}

#[cfg(test)]
mod test_queued {
	use super::*;

	#[test]
	fn prime_skill() {
		let mut queued = QueuedSkill {
			skill: Skill::default(),
			skill_mode: SkillMode::Hold,
			..default()
		};
		queued.release_skill();

		assert_eq!(SkillMode::Release, queued.skill_mode);
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

#[cfg(test)]
impl Default for RunSkillBehavior {
	fn default() -> Self {
		use crate::behaviors::spawn_skill::SpawnSkill;

		Self::OnActive(SkillBehaviorConfig::from_shape(SpawnSkill::NO_SHAPE))
	}
}

impl SpawnSkillBehavior for RunSkillBehavior {
	fn spawn_on(&self) -> SpawnOn {
		match self {
			RunSkillBehavior::OnActive(skill) => skill.spawn_on,
			RunSkillBehavior::OnAim(skill) => skill.spawn_on,
		}
	}

	fn spawn<TEffects, TSkillBehaviors>(
		&self,
		commands: &mut ZyheedaCommands,
		caster: &SkillCaster,
		spawner: SkillSpawner,
		target: &SkillTarget,
	) -> OnSkillStop
	where
		TEffects: HandlesAllPhysicalEffects + 'static,
		TSkillBehaviors: HandlesSkillBehaviors + 'static,
	{
		match self {
			RunSkillBehavior::OnActive(conf) => {
				spawn::<TEffects, TSkillBehaviors>(conf, commands, caster, spawner, target)
			}
			RunSkillBehavior::OnAim(conf) => {
				spawn::<TEffects, TSkillBehaviors>(conf, commands, caster, spawner, target)
			}
		}
	}
}

fn spawn<TEffects, TSkillBehaviors>(
	behavior: &SkillBehaviorConfig,
	commands: &mut ZyheedaCommands,
	caster: &SkillCaster,
	spawner: SkillSpawner,
	target: &SkillTarget,
) -> OnSkillStop
where
	TEffects: HandlesAllPhysicalEffects + 'static,
	TSkillBehaviors: HandlesSkillBehaviors + 'static,
{
	let shape = behavior.spawn_shape::<TSkillBehaviors>(commands, caster, spawner, target);

	if let Some(mut contact) = commands.get_mut(&shape.contact) {
		behavior.start_contact_behavior::<TEffects>(&mut contact, caster, target);
	};

	if let Some(mut projection) = commands.get_mut(&shape.projection) {
		behavior.start_projection_behavior::<TEffects>(&mut projection, caster, target);
	};

	shape.on_skill_stop
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		behaviors::{attach_skill_effect::AttachEffect, spawn_skill::SpawnSkill},
		traits::skill_builder::SkillShape,
	};
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{
		attributes::health::Health,
		components::{outdated::Outdated, persistent_entity::PersistentEntity},
		tools::collider_info::ColliderInfo,
		traits::{
			handles_physics::{Effect, HandlesPhysicalEffect},
			handles_skill_behaviors::{Contact, HoldSkills, Projection, SkillEntities, SkillRoot},
			thread_safe::ThreadSafe,
		},
		zyheeda_commands::ZyheedaEntityCommands,
	};
	use std::array::IntoIter;
	use testing::SingleThreadedApp;

	#[derive(Component, Debug, PartialEq)]
	struct _Args {
		caster: SkillCaster,
		target: SkillTarget,
	}

	#[derive(Component)]
	struct _Contact;

	#[derive(Component)]
	struct _Projection;

	#[derive(Component)]
	struct _SkillUsage;

	impl HoldSkills for _SkillUsage {
		type Iter<'a> = IntoIter<SlotKey, 0>;

		fn holding(&self) -> Self::Iter<'_> {
			[].into_iter()
		}

		fn started_holding(&self) -> Self::Iter<'_> {
			[].into_iter()
		}
	}

	struct _HandlesEffects;

	impl<T> HandlesPhysicalEffect<T> for _HandlesEffects
	where
		T: Effect + ThreadSafe,
	{
		type TEffectComponent = _Effect;
		type TAffectedComponent = _Affected;

		fn into_effect_component(_: T) -> _Effect {
			_Effect
		}
	}

	#[derive(Component)]
	struct _Effect;

	#[derive(Component)]
	struct _Affected;

	impl GetProperty<Health> for _Affected {
		fn get_property(&self) -> Health {
			panic!("NOT USED")
		}
	}

	struct _HandlesSkillBehaviors;

	impl HandlesSkillBehaviors for _HandlesSkillBehaviors {
		type TSkillContact = _Contact;
		type TSkillProjection = _Projection;
		type TSkillUsage = _SkillUsage;

		fn spawn_skill(commands: &mut ZyheedaCommands, _: Contact, _: Projection) -> SkillEntities {
			SkillEntities {
				root: SkillRoot {
					entity: commands.spawn(()).id(),
					persistent_entity: PersistentEntity::default(),
				},
				contact: commands.spawn(_Contact).id(),
				projection: commands.spawn(_Projection).id(),
			}
		}
	}

	fn effect_fn(e: &mut ZyheedaEntityCommands, c: &SkillCaster, t: &SkillTarget) {
		e.try_insert(_Args {
			caster: *c,
			target: *t,
		});
	}

	fn get_target() -> SkillTarget {
		SkillTarget {
			ray: Ray3d::new(
				Vec3::new(1., 2., 3.),
				Dir3::new_unchecked(Vec3::new(4., 5., 6.).normalize()),
			),
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

	fn execute_callback<TCallback>(In(mut callback): In<TCallback>, mut cmd: ZyheedaCommands)
	where
		TCallback: FnMut(&mut ZyheedaCommands),
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
	fn spawn_skill_contact_entity_on_active() -> Result<(), RunSystemError> {
		let mut app = setup();
		let behavior = RunSkillBehavior::OnActive(
			SkillBehaviorConfig::from_shape(SpawnSkill::Fn(|cmd, caster, _, target| SkillShape {
				contact: cmd
					.spawn(_Args {
						caster: *caster,
						target: *target,
					})
					.id(),
				projection: cmd.spawn(()).id(),
				on_skill_stop: OnSkillStop::Ignore,
			}))
			.spawning_on(SpawnOn::Slot),
		);
		let caster = SkillCaster(PersistentEntity::default());
		let spawner = SkillSpawner::Neutral;
		let target = get_target();

		app.world_mut()
			.run_system_once_with(execute_callback, move |cmd| {
				behavior.spawn::<_HandlesEffects, _HandlesSkillBehaviors>(
					cmd, &caster, spawner, &target,
				);
			})?;

		assert_eq!(
			vec![&_Args { caster, target }],
			spawned_args(&app, no_filter)
		);
		Ok(())
	}

	#[test]
	fn spawn_skill_contact_entity_on_active_centered() -> Result<(), RunSystemError> {
		let mut app = setup();
		let behavior = RunSkillBehavior::OnActive(
			SkillBehaviorConfig::from_shape(SpawnSkill::Fn(|cmd, caster, _, target| SkillShape {
				contact: cmd
					.spawn(_Args {
						caster: *caster,
						target: *target,
					})
					.id(),
				projection: cmd.spawn(()).id(),
				on_skill_stop: OnSkillStop::Ignore,
			}))
			.spawning_on(SpawnOn::Center),
		);
		let caster = SkillCaster(PersistentEntity::default());
		let spawner = SkillSpawner::Neutral;
		let target = get_target();

		app.world_mut()
			.run_system_once_with(execute_callback, move |cmd| {
				behavior.spawn::<_HandlesEffects, _HandlesSkillBehaviors>(
					cmd, &caster, spawner, &target,
				);
			})?;

		assert_eq!(
			vec![&_Args { caster, target }],
			spawned_args(&app, no_filter)
		);
		Ok(())
	}

	#[test]
	fn apply_contact_behavior_on_active() -> Result<(), RunSystemError> {
		fn shape(
			cmd: &mut ZyheedaCommands,
			_: &SkillCaster,
			_: SkillSpawner,
			_: &SkillTarget,
		) -> SkillShape {
			SkillShape {
				contact: cmd.spawn(_Contact).id(),
				projection: cmd.spawn(()).id(),
				on_skill_stop: OnSkillStop::Ignore,
			}
		}

		let mut app = setup();
		let behavior = RunSkillBehavior::OnActive(
			SkillBehaviorConfig::from_shape(SpawnSkill::Fn(shape))
				.with_contact_effects(vec![AttachEffect::Fn(effect_fn)]),
		);
		let caster = SkillCaster(PersistentEntity::default());
		let spawner = SkillSpawner::Neutral;
		let target = get_target();

		app.world_mut()
			.run_system_once_with(execute_callback, move |cmd| {
				behavior.spawn::<_HandlesEffects, _HandlesSkillBehaviors>(
					cmd, &caster, spawner, &target,
				);
			})?;

		assert_eq!(
			vec![&_Args { caster, target }],
			spawned_args(&app, filter::<_Contact>)
		);
		Ok(())
	}

	#[test]
	fn spawn_skill_projection_entity_on_active() -> Result<(), RunSystemError> {
		let mut app = setup();
		let behavior = RunSkillBehavior::OnActive(SkillBehaviorConfig::from_shape(SpawnSkill::Fn(
			|cmd, caster, _, target| SkillShape {
				contact: cmd
					.spawn(_Args {
						caster: *caster,
						target: *target,
					})
					.id(),
				projection: cmd.spawn(()).id(),
				on_skill_stop: OnSkillStop::Ignore,
			},
		)));
		let caster = SkillCaster(PersistentEntity::default());
		let spawner = SkillSpawner::Neutral;
		let target = get_target();

		app.world_mut()
			.run_system_once_with(execute_callback, move |cmd| {
				behavior.spawn::<_HandlesEffects, _HandlesSkillBehaviors>(
					cmd, &caster, spawner, &target,
				);
			})?;

		assert_eq!(
			vec![&_Args { caster, target }],
			spawned_args(&app, no_filter)
		);
		Ok(())
	}

	#[test]
	fn apply_projection_behavior_on_active() -> Result<(), RunSystemError> {
		fn shape(
			cmd: &mut ZyheedaCommands,
			_: &SkillCaster,
			_: SkillSpawner,
			_: &SkillTarget,
		) -> SkillShape {
			SkillShape {
				contact: cmd.spawn(()).id(),
				projection: cmd.spawn(_Projection).id(),
				on_skill_stop: OnSkillStop::Ignore,
			}
		}

		let mut app = setup();
		let behavior = RunSkillBehavior::OnActive(
			SkillBehaviorConfig::from_shape(SpawnSkill::Fn(shape))
				.with_projection_effects(vec![AttachEffect::Fn(effect_fn)]),
		);
		let caster = SkillCaster(PersistentEntity::default());
		let spawner = SkillSpawner::Neutral;
		let target = get_target();

		app.world_mut()
			.run_system_once_with(execute_callback, move |cmd| {
				behavior.spawn::<_HandlesEffects, _HandlesSkillBehaviors>(
					cmd, &caster, spawner, &target,
				);
			})?;

		let spawn_args = app
			.world()
			.iter_entities()
			.filter(|e| e.contains::<_Projection>())
			.filter_map(|e| e.get::<_Args>())
			.collect::<Vec<_>>();

		assert_eq!(vec![&_Args { caster, target }], spawn_args);
		Ok(())
	}

	#[test]
	fn spawn_skill_contact_entity_on_aim() -> Result<(), RunSystemError> {
		let mut app = setup();
		let behavior = RunSkillBehavior::OnAim(
			SkillBehaviorConfig::from_shape(SpawnSkill::Fn(|cmd, caster, _, target| SkillShape {
				contact: cmd
					.spawn(_Args {
						caster: *caster,
						target: *target,
					})
					.id(),
				projection: cmd.spawn(()).id(),
				on_skill_stop: OnSkillStop::Ignore,
			}))
			.spawning_on(SpawnOn::Slot),
		);
		let caster = SkillCaster(PersistentEntity::default());
		let spawner = SkillSpawner::Neutral;
		let target = get_target();

		app.world_mut()
			.run_system_once_with(execute_callback, move |cmd| {
				behavior.spawn::<_HandlesEffects, _HandlesSkillBehaviors>(
					cmd, &caster, spawner, &target,
				);
			})?;

		assert_eq!(
			vec![&_Args { caster, target }],
			spawned_args(&app, no_filter)
		);
		Ok(())
	}

	#[test]
	fn spawn_skill_contact_entity_on_aim_centered() -> Result<(), RunSystemError> {
		let mut app = setup();
		let behavior = RunSkillBehavior::OnAim(
			SkillBehaviorConfig::from_shape(SpawnSkill::Fn(|cmd, caster, _, target| SkillShape {
				contact: cmd
					.spawn(_Args {
						caster: *caster,
						target: *target,
					})
					.id(),
				projection: cmd.spawn(()).id(),
				on_skill_stop: OnSkillStop::Ignore,
			}))
			.spawning_on(SpawnOn::Center),
		);
		let caster = SkillCaster(PersistentEntity::default());
		let spawner = SkillSpawner::Neutral;
		let target = get_target();

		app.world_mut()
			.run_system_once_with(execute_callback, move |cmd| {
				behavior.spawn::<_HandlesEffects, _HandlesSkillBehaviors>(
					cmd, &caster, spawner, &target,
				);
			})?;

		assert_eq!(
			vec![&_Args { caster, target }],
			spawned_args(&app, no_filter)
		);
		Ok(())
	}

	#[test]
	fn apply_contact_behavior_on_aim() -> Result<(), RunSystemError> {
		#[derive(Component)]
		struct _Contact;

		fn shape(
			cmd: &mut ZyheedaCommands,
			_: &SkillCaster,
			_: SkillSpawner,
			_: &SkillTarget,
		) -> SkillShape {
			SkillShape {
				contact: cmd.spawn(_Contact).id(),
				projection: cmd.spawn(()).id(),
				on_skill_stop: OnSkillStop::Ignore,
			}
		}

		let mut app = setup();
		let behavior = RunSkillBehavior::OnAim(
			SkillBehaviorConfig::from_shape(SpawnSkill::Fn(shape))
				.with_contact_effects(vec![AttachEffect::Fn(effect_fn)]),
		);
		let caster = SkillCaster(PersistentEntity::default());
		let spawner = SkillSpawner::Neutral;
		let target = get_target();

		app.world_mut()
			.run_system_once_with(execute_callback, move |cmd| {
				behavior.spawn::<_HandlesEffects, _HandlesSkillBehaviors>(
					cmd, &caster, spawner, &target,
				);
			})?;

		let spawn_args = app
			.world()
			.iter_entities()
			.filter(|e| e.contains::<_Contact>())
			.filter_map(|e| e.get::<_Args>())
			.collect::<Vec<_>>();

		assert_eq!(vec![&_Args { caster, target }], spawn_args);
		Ok(())
	}

	#[test]
	fn spawn_skill_projection_entity_on_aim() -> Result<(), RunSystemError> {
		let mut app = setup();
		let behavior = RunSkillBehavior::OnAim(SkillBehaviorConfig::from_shape(SpawnSkill::Fn(
			|cmd, caster, _, target| SkillShape {
				contact: cmd
					.spawn(_Args {
						caster: *caster,
						target: *target,
					})
					.id(),
				projection: cmd.spawn(()).id(),
				on_skill_stop: OnSkillStop::Ignore,
			},
		)));
		let caster = SkillCaster(PersistentEntity::default());
		let spawner = SkillSpawner::Neutral;
		let target = get_target();

		app.world_mut()
			.run_system_once_with(execute_callback, move |cmd| {
				behavior.spawn::<_HandlesEffects, _HandlesSkillBehaviors>(
					cmd, &caster, spawner, &target,
				);
			})?;

		assert_eq!(
			vec![&_Args { caster, target }],
			spawned_args(&app, no_filter)
		);
		Ok(())
	}

	#[test]
	fn apply_projection_behavior_on_aim() -> Result<(), RunSystemError> {
		#[derive(Component)]
		struct _Projection;

		fn shape(
			cmd: &mut ZyheedaCommands,
			_: &SkillCaster,
			_: SkillSpawner,
			_: &SkillTarget,
		) -> SkillShape {
			SkillShape {
				contact: cmd.spawn(()).id(),
				projection: cmd.spawn(_Projection).id(),
				on_skill_stop: OnSkillStop::Ignore,
			}
		}

		let mut app = setup();
		let behavior = RunSkillBehavior::OnAim(
			SkillBehaviorConfig::from_shape(SpawnSkill::Fn(shape))
				.with_projection_effects(vec![AttachEffect::Fn(effect_fn)]),
		);
		let caster = SkillCaster(PersistentEntity::default());
		let spawner = SkillSpawner::Neutral;
		let target = get_target();

		app.world_mut()
			.run_system_once_with(execute_callback, move |cmd| {
				behavior.spawn::<_HandlesEffects, _HandlesSkillBehaviors>(
					cmd, &caster, spawner, &target,
				);
			})?;

		let spawn_args = app
			.world()
			.iter_entities()
			.filter(|e| e.contains::<_Projection>())
			.filter_map(|e| e.get::<_Args>())
			.collect::<Vec<_>>();

		assert_eq!(vec![&_Args { caster, target }], spawn_args);
		Ok(())
	}
}
