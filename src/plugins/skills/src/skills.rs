pub(crate) mod dto;
pub(crate) mod lifetime_definition;

use crate::{
	behaviors::{
		SkillBehaviorConfig,
		SkillCaster,
		SkillSpawner,
		build_skill_shape::{BuildSkillShape, OnSkillStop},
		spawn_on::SpawnOn,
	},
	components::SkillTarget,
	traits::{Matches, Prime, spawn_skill_behavior::SpawnSkillBehavior},
};
use bevy::prelude::*;
use common::{
	tools::{
		action_key::slot::SlotKey,
		item_type::CompatibleItems,
		skill_description::SkillToken,
		skill_icon::SkillIcon,
	},
	traits::{
		accessors::get::{Getter, GetterRef},
		animation::Animation,
		handles_custom_assets::AssetFolderPath,
		handles_effect::HandlesAllEffects,
		handles_lifetime::HandlesLifetime,
		handles_localization::Token,
		handles_skill_behaviors::HandlesSkillBehaviors,
		inspect_able::InspectAble,
		load_asset::Path,
	},
};
use serde::{Deserialize, Serialize};
use std::{
	fmt::{Display, Formatter, Result as FmtResult},
	time::Duration,
};

#[derive(PartialEq, Debug, Clone)]
pub struct SkillAnimation {
	pub(crate) top_hand_left: Animation,
	pub(crate) top_hand_right: Animation,
	pub(crate) btm_hand_left: Animation,
	pub(crate) btm_hand_right: Animation,
}

#[derive(PartialEq, Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub enum AnimationStrategy {
	#[default]
	None,
	DoNotAnimate,
	Animate,
}

#[derive(PartialEq, Debug, Default, Clone, TypePath, Asset)]
pub struct Skill {
	pub token: Token,
	pub cast_time: Duration,
	pub animation: AnimationStrategy,
	pub behavior: RunSkillBehavior,
	pub compatible_items: CompatibleItems,
	pub icon: Option<Handle<Image>>,
}

impl Display for Skill {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		let Token(token) = &self.token;
		match token.as_str() {
			"" => write!(f, "Skill(<no token>)"),
			name => write!(f, "Skill({})", name),
		}
	}
}

impl AssetFolderPath for Skill {
	fn asset_folder_path() -> Path {
		Path::from("skills")
	}
}

impl InspectAble<SkillToken> for Skill {
	fn get_inspect_able_field(&self) -> &Token {
		&self.token
	}
}

impl InspectAble<SkillIcon> for Skill {
	fn get_inspect_able_field(&self) -> &Option<Handle<Image>> {
		&self.icon
	}
}

impl GetterRef<Option<Handle<Image>>> for Skill {
	fn get(&self) -> &Option<Handle<Image>> {
		&self.icon
	}
}

impl GetterRef<CompatibleItems> for Skill {
	fn get(&self) -> &CompatibleItems {
		&self.compatible_items
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

impl Getter<SlotKey> for QueuedSkill {
	fn get(&self) -> SlotKey {
		self.slot_key
	}
}

impl InspectAble<SkillToken> for QueuedSkill {
	fn get_inspect_able_field(&self) -> &Token {
		&self.skill.token
	}
}

impl InspectAble<SkillIcon> for QueuedSkill {
	fn get_inspect_able_field(&self) -> &Option<Handle<Image>> {
		&self.skill.icon
	}
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

	fn spawn<TLifetimes, TEffects, TSkillBehaviors>(
		&self,
		commands: &mut Commands,
		caster: &SkillCaster,
		spawner: &SkillSpawner,
		target: &SkillTarget,
	) -> OnSkillStop
	where
		TLifetimes: HandlesLifetime + 'static,
		TEffects: HandlesAllEffects + 'static,
		TSkillBehaviors: HandlesSkillBehaviors + 'static,
	{
		match self {
			RunSkillBehavior::OnActive(conf) => spawn::<TLifetimes, TEffects, TSkillBehaviors>(
				conf, commands, caster, spawner, target,
			),
			RunSkillBehavior::OnAim(conf) => spawn::<TLifetimes, TEffects, TSkillBehaviors>(
				conf, commands, caster, spawner, target,
			),
		}
	}
}

fn spawn<TLifetimes, TEffects, TSkillBehaviors>(
	behavior: &SkillBehaviorConfig,
	commands: &mut Commands,
	caster: &SkillCaster,
	spawner: &SkillSpawner,
	target: &SkillTarget,
) -> OnSkillStop
where
	TLifetimes: HandlesLifetime + 'static,
	TEffects: HandlesAllEffects + 'static,
	TSkillBehaviors: HandlesSkillBehaviors + 'static,
{
	let shape =
		behavior.spawn_shape::<TLifetimes, TSkillBehaviors>(commands, caster, spawner, target);

	if let Ok(mut contact) = commands.get_entity(shape.contact) {
		behavior.start_contact_behavior::<TEffects>(&mut contact, caster, spawner, target);
	};

	if let Ok(mut projection) = commands.get_entity(shape.projection) {
		behavior.start_projection_behavior::<TEffects>(&mut projection, caster, spawner, target);
	};

	shape.on_skill_stop
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{behaviors::start_behavior::SkillBehavior, traits::skill_builder::SkillShape};
	use bevy::ecs::system::{EntityCommands, RunSystemError, RunSystemOnce};
	use common::{
		components::Outdated,
		test_tools::utils::SingleThreadedApp,
		tools::collider_info::ColliderInfo,
		traits::{
			handles_effect::HandlesEffect,
			handles_skill_behaviors::{Integrity, Motion, ProjectionOffset, Shape},
		},
	};

	#[derive(Component, Debug, PartialEq)]
	struct _Args {
		caster: SkillCaster,
		spawner: SkillSpawner,
		target: SkillTarget,
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
		type TTarget = ();
		type TEffectComponent = _Effect;

		fn effect(_: T) -> _Effect {
			_Effect
		}

		fn attribute(_: Self::TTarget) -> impl Bundle {}
	}

	#[derive(Component)]
	struct _Effect;

	struct _HandlesSkillBehaviors;

	impl HandlesSkillBehaviors for _HandlesSkillBehaviors {
		type TSkillContact = _Contact;
		type TSkillProjection = _Projection;

		fn skill_contact(_: Shape, _: Integrity, _: Motion) -> Self::TSkillContact {
			_Contact
		}

		fn skill_projection(_: Shape, _: Option<ProjectionOffset>) -> Self::TSkillProjection {
			_Projection
		}
	}

	fn behavior(e: &mut EntityCommands, c: &SkillCaster, s: &SkillSpawner, t: &SkillTarget) {
		e.try_insert(_Args {
			caster: *c,
			spawner: *s,
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
	fn spawn_skill_contact_entity_on_active() -> Result<(), RunSystemError> {
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

		app.world_mut()
			.run_system_once_with(execute_callback, move |cmd| {
				behavior.spawn::<_HandlesLifetime, _HandlesEffects, _HandlesSkillBehaviors>(
					cmd, &caster, &spawner, &target,
				);
			})?;

		assert_eq!(
			vec![&_Args {
				caster,
				spawner,
				target
			}],
			spawned_args(&app, no_filter)
		);
		Ok(())
	}

	#[test]
	fn spawn_skill_contact_entity_on_active_centered() -> Result<(), RunSystemError> {
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

		app.world_mut()
			.run_system_once_with(execute_callback, move |cmd| {
				behavior.spawn::<_HandlesLifetime, _HandlesEffects, _HandlesSkillBehaviors>(
					cmd, &caster, &spawner, &target,
				);
			})?;

		assert_eq!(
			vec![&_Args {
				caster,
				spawner,
				target
			}],
			spawned_args(&app, no_filter)
		);
		Ok(())
	}

	#[test]
	fn apply_contact_behavior_on_active() -> Result<(), RunSystemError> {
		fn shape(
			cmd: &mut Commands,
			_: &SkillCaster,
			_: &SkillSpawner,
			_: &SkillTarget,
		) -> SkillShape {
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

		app.world_mut()
			.run_system_once_with(execute_callback, move |cmd| {
				behavior.spawn::<_HandlesLifetime, _HandlesEffects, _HandlesSkillBehaviors>(
					cmd, &caster, &spawner, &target,
				);
			})?;

		assert_eq!(
			vec![&_Args {
				caster,
				spawner,
				target
			}],
			spawned_args(&app, filter::<_Contact>)
		);
		Ok(())
	}

	#[test]
	fn spawn_skill_projection_entity_on_active() -> Result<(), RunSystemError> {
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

		app.world_mut()
			.run_system_once_with(execute_callback, move |cmd| {
				behavior.spawn::<_HandlesLifetime, _HandlesEffects, _HandlesSkillBehaviors>(
					cmd, &caster, &spawner, &target,
				);
			})?;

		assert_eq!(
			vec![&_Args {
				caster,
				spawner,
				target
			}],
			spawned_args(&app, no_filter)
		);
		Ok(())
	}

	#[test]
	fn apply_projection_behavior_on_active() -> Result<(), RunSystemError> {
		fn shape(
			cmd: &mut Commands,
			_: &SkillCaster,
			_: &SkillSpawner,
			_: &SkillTarget,
		) -> SkillShape {
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

		app.world_mut()
			.run_system_once_with(execute_callback, move |cmd| {
				behavior.spawn::<_HandlesLifetime, _HandlesEffects, _HandlesSkillBehaviors>(
					cmd, &caster, &spawner, &target,
				);
			})?;

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
		Ok(())
	}

	#[test]
	fn spawn_skill_contact_entity_on_aim() -> Result<(), RunSystemError> {
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

		app.world_mut()
			.run_system_once_with(execute_callback, move |cmd| {
				behavior.spawn::<_HandlesLifetime, _HandlesEffects, _HandlesSkillBehaviors>(
					cmd, &caster, &spawner, &target,
				);
			})?;

		assert_eq!(
			vec![&_Args {
				caster,
				spawner,
				target
			}],
			spawned_args(&app, no_filter)
		);
		Ok(())
	}

	#[test]
	fn spawn_skill_contact_entity_on_aim_centered() -> Result<(), RunSystemError> {
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

		app.world_mut()
			.run_system_once_with(execute_callback, move |cmd| {
				behavior.spawn::<_HandlesLifetime, _HandlesEffects, _HandlesSkillBehaviors>(
					cmd, &caster, &spawner, &target,
				);
			})?;

		assert_eq!(
			vec![&_Args {
				caster,
				spawner,
				target
			}],
			spawned_args(&app, no_filter)
		);
		Ok(())
	}

	#[test]
	fn apply_contact_behavior_on_aim() -> Result<(), RunSystemError> {
		#[derive(Component)]
		struct _Contact;

		fn shape(
			cmd: &mut Commands,
			_: &SkillCaster,
			_: &SkillSpawner,
			_: &SkillTarget,
		) -> SkillShape {
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

		app.world_mut()
			.run_system_once_with(execute_callback, move |cmd| {
				behavior.spawn::<_HandlesLifetime, _HandlesEffects, _HandlesSkillBehaviors>(
					cmd, &caster, &spawner, &target,
				);
			})?;

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
		Ok(())
	}

	#[test]
	fn spawn_skill_projection_entity_on_aim() -> Result<(), RunSystemError> {
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

		app.world_mut()
			.run_system_once_with(execute_callback, move |cmd| {
				behavior.spawn::<_HandlesLifetime, _HandlesEffects, _HandlesSkillBehaviors>(
					cmd, &caster, &spawner, &target,
				);
			})?;

		assert_eq!(
			vec![&_Args {
				caster,
				spawner,
				target
			}],
			spawned_args(&app, no_filter)
		);
		Ok(())
	}

	#[test]
	fn apply_projection_behavior_on_aim() -> Result<(), RunSystemError> {
		#[derive(Component)]
		struct _Projection;

		fn shape(
			cmd: &mut Commands,
			_: &SkillCaster,
			_: &SkillSpawner,
			_: &SkillTarget,
		) -> SkillShape {
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

		app.world_mut()
			.run_system_once_with(execute_callback, move |cmd| {
				behavior.spawn::<_HandlesLifetime, _HandlesEffects, _HandlesSkillBehaviors>(
					cmd, &caster, &spawner, &target,
				);
			})?;

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
		Ok(())
	}
}
