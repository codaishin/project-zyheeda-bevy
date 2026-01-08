pub(crate) mod dto;
pub(crate) mod lifetime_definition;

use crate::{
	behaviors::{
		SkillBehaviorConfig,
		spawn_skill::{OnSkillStop, SpawnOn},
	},
	traits::{ReleaseSkill, spawn_loadout_skill::SpawnLoadoutSkill},
};
use bevy::prelude::*;
use common::{
	tools::{
		action_key::slot::SlotKey,
		item_type::{CompatibleItems, ItemType},
		path::Path,
	},
	traits::{
		accessors::get::{GetMut, GetProperty},
		handles_custom_assets::AssetFolderPath,
		handles_loadout::skills::{GetSkillId, SkillIcon, SkillToken},
		handles_localization::Token,
		handles_physics::HandlesAllPhysicalEffects,
		handles_skill_physics::{HandlesNewPhysicalSkill, SkillCaster, SkillSpawner, SkillTarget},
	},
	zyheeda_commands::ZyheedaCommands,
};
use serde::{Deserialize, Serialize};
use std::{
	collections::HashSet,
	fmt::{Display, Formatter, Result as FmtResult},
	time::Duration,
};
use uuid::Uuid;
#[cfg(test)]
use uuid::uuid;

#[derive(PartialEq, Debug, Clone, TypePath, Asset)]
#[cfg_attr(test, derive(Default))]
pub struct Skill {
	pub(crate) id: SkillId,
	pub(crate) token: Token,
	pub(crate) cast_time: Duration,
	pub(crate) animate: bool,
	pub(crate) behavior: RunSkillBehavior,
	pub(crate) compatible_items: CompatibleItems,
	pub(crate) icon: Handle<Image>,
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

impl GetSkillId<SkillId> for Skill {
	fn get_skill_id(&self) -> SkillId {
		self.id
	}
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SkillId(pub(crate) Uuid);

#[cfg(test)]
impl SkillId {
	const DEFAULT_ID: SkillId = SkillId(uuid!("9443883c-3972-43da-a2d7-0a013f16d564"));
}

#[cfg(test)]
impl Default for SkillId {
	fn default() -> Self {
		Self::DEFAULT_ID
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

impl SpawnLoadoutSkill for RunSkillBehavior {
	fn spawn_on(&self) -> SpawnOn {
		match self {
			RunSkillBehavior::OnActive(skill) => skill.spawn_on,
			RunSkillBehavior::OnAim(skill) => skill.spawn_on,
		}
	}

	fn spawn<TPhysics>(
		&self,
		commands: &mut ZyheedaCommands,
		caster: SkillCaster,
		spawner: SkillSpawner,
		target: SkillTarget,
	) -> OnSkillStop
	where
		TPhysics: HandlesAllPhysicalEffects + HandlesNewPhysicalSkill + 'static,
	{
		match self {
			RunSkillBehavior::OnActive(conf) => {
				spawn::<TPhysics>(conf, commands, caster, spawner, target)
			}
			RunSkillBehavior::OnAim(conf) => {
				spawn::<TPhysics>(conf, commands, caster, spawner, target)
			}
		}
	}
}

fn spawn<TPhysics>(
	behavior: &SkillBehaviorConfig,
	commands: &mut ZyheedaCommands,
	caster: SkillCaster,
	spawner: SkillSpawner,
	target: SkillTarget,
) -> OnSkillStop
where
	TPhysics: HandlesAllPhysicalEffects + HandlesNewPhysicalSkill + 'static,
{
	let shape = behavior.spawn_shape::<TPhysics>(commands, caster, spawner, target);

	if let Some(mut contact) = commands.get_mut(&shape.contact) {
		behavior.start_contact_behavior::<TPhysics>(&mut contact, caster, target);
	};

	if let Some(mut projection) = commands.get_mut(&shape.projection) {
		behavior.start_projection_behavior::<TPhysics>(&mut projection, caster, target);
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
	use bevy::ecs::system::{RunSystemError, RunSystemOnce, SystemParam};
	use common::{
		attributes::health::Health,
		components::persistent_entity::PersistentEntity,
		traits::{
			handles_physics::{Effect as EffectTrait, HandlesPhysicalEffect},
			handles_skill_physics::{
				Contact,
				Effect,
				HandlesNewPhysicalSkill,
				Projection,
				Skill,
				SkillEntities,
				SkillRoot,
				Spawn,
			},
			thread_safe::ThreadSafe,
		},
		zyheeda_commands::ZyheedaEntityCommands,
	};
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

	#[derive(SystemParam)]
	struct _SkillSpawner;

	impl Spawn for _SkillSpawner {
		type TSkill<'c>
			= _SpawnedSkill
		where
			Self: 'c;

		fn spawn(&mut self, _: Contact, _: Projection) -> Self::TSkill<'_> {
			panic!("SHOULD NOT BE CALLED")
		}
	}

	struct _SpawnedSkill;

	impl Skill for _SpawnedSkill {
		fn root(&self) -> PersistentEntity {
			panic!("SHOULD NOT BE CALLED")
		}

		fn insert_on_root<T>(&mut self, _: T)
		where
			T: Bundle,
		{
			panic!("SHOULD NOT BE CALLED")
		}

		fn insert_on_contact(&mut self, _: Effect) {
			panic!("SHOULD NOT BE CALLED")
		}

		fn insert_on_projection(&mut self, _: Effect) {
			panic!("SHOULD NOT BE CALLED")
		}
	}

	struct _HandlesPhysics;

	impl<T> HandlesPhysicalEffect<T> for _HandlesPhysics
	where
		T: EffectTrait + ThreadSafe,
	{
		type TEffectComponent = _Effect;
		type TAffectedComponent = _Affected;

		fn into_effect_component(_: T) -> _Effect {
			_Effect
		}
	}

	impl HandlesNewPhysicalSkill for _HandlesPhysics {
		type TSkillSpawnerMut<'w, 's> = _SkillSpawner;

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

	#[derive(Component)]
	struct _Effect;

	#[derive(Component)]
	struct _Affected;

	impl GetProperty<Health> for _Affected {
		fn get_property(&self) -> Health {
			panic!("NOT USED")
		}
	}

	fn effect_fn(e: &mut ZyheedaEntityCommands, caster: SkillCaster, target: SkillTarget) {
		e.try_insert(_Args { caster, target });
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
				contact: cmd.spawn(_Args { caster, target }).id(),
				projection: cmd.spawn(()).id(),
				on_skill_stop: OnSkillStop::Ignore,
			}))
			.spawning_on(SpawnOn::Slot),
		);
		let caster = SkillCaster(PersistentEntity::default());
		let spawner = SkillSpawner::Neutral;
		let target = SkillTarget::Ground(Vec3::new(1., 2., 3.));

		app.world_mut()
			.run_system_once_with(execute_callback, move |cmd| {
				behavior.spawn::<_HandlesPhysics>(cmd, caster, spawner, target);
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
				contact: cmd.spawn(_Args { caster, target }).id(),
				projection: cmd.spawn(()).id(),
				on_skill_stop: OnSkillStop::Ignore,
			}))
			.spawning_on(SpawnOn::Center),
		);
		let caster = SkillCaster(PersistentEntity::default());
		let spawner = SkillSpawner::Neutral;
		let target = SkillTarget::Ground(Vec3::new(1., 2., 3.));

		app.world_mut()
			.run_system_once_with(execute_callback, move |cmd| {
				behavior.spawn::<_HandlesPhysics>(cmd, caster, spawner, target);
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
			_: SkillCaster,
			_: SkillSpawner,
			_: SkillTarget,
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
		let target = SkillTarget::Ground(Vec3::new(1., 2., 3.));

		app.world_mut()
			.run_system_once_with(execute_callback, move |cmd| {
				behavior.spawn::<_HandlesPhysics>(cmd, caster, spawner, target);
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
				contact: cmd.spawn(_Args { caster, target }).id(),
				projection: cmd.spawn(()).id(),
				on_skill_stop: OnSkillStop::Ignore,
			},
		)));
		let caster = SkillCaster(PersistentEntity::default());
		let spawner = SkillSpawner::Neutral;
		let target = SkillTarget::Ground(Vec3::new(1., 2., 3.));

		app.world_mut()
			.run_system_once_with(execute_callback, move |cmd| {
				behavior.spawn::<_HandlesPhysics>(cmd, caster, spawner, target);
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
			_: SkillCaster,
			_: SkillSpawner,
			_: SkillTarget,
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
		let target = SkillTarget::Ground(Vec3::new(1., 2., 3.));

		app.world_mut()
			.run_system_once_with(execute_callback, move |cmd| {
				behavior.spawn::<_HandlesPhysics>(cmd, caster, spawner, target);
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
				contact: cmd.spawn(_Args { caster, target }).id(),
				projection: cmd.spawn(()).id(),
				on_skill_stop: OnSkillStop::Ignore,
			}))
			.spawning_on(SpawnOn::Slot),
		);
		let caster = SkillCaster(PersistentEntity::default());
		let spawner = SkillSpawner::Neutral;
		let target = SkillTarget::Ground(Vec3::new(1., 2., 3.));

		app.world_mut()
			.run_system_once_with(execute_callback, move |cmd| {
				behavior.spawn::<_HandlesPhysics>(cmd, caster, spawner, target);
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
				contact: cmd.spawn(_Args { caster, target }).id(),
				projection: cmd.spawn(()).id(),
				on_skill_stop: OnSkillStop::Ignore,
			}))
			.spawning_on(SpawnOn::Center),
		);
		let caster = SkillCaster(PersistentEntity::default());
		let spawner = SkillSpawner::Neutral;
		let target = SkillTarget::Ground(Vec3::new(1., 2., 3.));

		app.world_mut()
			.run_system_once_with(execute_callback, move |cmd| {
				behavior.spawn::<_HandlesPhysics>(cmd, caster, spawner, target);
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
			_: SkillCaster,
			_: SkillSpawner,
			_: SkillTarget,
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
		let target = SkillTarget::Ground(Vec3::new(1., 2., 3.));

		app.world_mut()
			.run_system_once_with(execute_callback, move |cmd| {
				behavior.spawn::<_HandlesPhysics>(cmd, caster, spawner, target);
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
				contact: cmd.spawn(_Args { caster, target }).id(),
				projection: cmd.spawn(()).id(),
				on_skill_stop: OnSkillStop::Ignore,
			},
		)));
		let caster = SkillCaster(PersistentEntity::default());
		let spawner = SkillSpawner::Neutral;
		let target = SkillTarget::Ground(Vec3::new(1., 2., 3.));

		app.world_mut()
			.run_system_once_with(execute_callback, move |cmd| {
				behavior.spawn::<_HandlesPhysics>(cmd, caster, spawner, target);
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
			_: SkillCaster,
			_: SkillSpawner,
			_: SkillTarget,
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
		let target = SkillTarget::Ground(Vec3::new(1., 2., 3.));

		app.world_mut()
			.run_system_once_with(execute_callback, move |cmd| {
				behavior.spawn::<_HandlesPhysics>(cmd, caster, spawner, target);
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
