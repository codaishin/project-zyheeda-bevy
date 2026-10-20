use crate::{components::self_skill_scale::SelfSkillScale, messages::RayEvent};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use bevy_rapier3d::prelude::*;
use common::prelude::*;
use std::marker::PhantomData;

pub(crate) struct Debug<TInput>(pub(super) PhantomData<TInput>);

impl<TInput> Debug<TInput>
where
	TInput: ThreadSafe + HandlesInput + SystemSetDefinition,
{
	fn display_events(
		mut collision_messages: MessageReader<CollisionEvent>,
		mut contact_force_messages: MessageReader<ContactForceEvent>,
		mut ray_cast_messages: MessageReader<RayEvent>,
	) {
		for collision_message in collision_messages.read() {
			info!("Received collision message: {collision_message:?}");
		}

		for contact_force_message in contact_force_messages.read() {
			info!("Received contact force message: {contact_force_message:?}");
		}

		for ray_cast_message in ray_cast_messages.read() {
			info!("Received ray cast message: {ray_cast_message:?}");
		}
	}

	fn set_collider_gizmos(
		mut gizmos: ResMut<ColliderGizmos>,
		input: Option<StaticSystemParam<TInput::TInput>>,
	) {
		let Some(input) = input else {
			return;
		};

		for (key, state) in input.get_all_input_states::<DebugKey>() {
			if key != DebugKey::RotateColliderDebug {
				continue;
			}

			if state != InputState::just_pressed() {
				continue;
			}

			gizmos.rotate();
		}
	}

	fn control_collider_debug(
		mut commands: ZyheedaCommands,
		gizmos: Res<ColliderGizmos>,
		colliders: Query<(Entity, &GlobalTransform, Option<&ColliderDebug>), With<Collider>>,
		agents: Query<&GlobalTransform, With<SelfSkillScale>>,
	) {
		for (entity, collider_transform, current_debug) in colliders {
			let debug = match *gizmos {
				ColliderGizmos::None => ColliderDebug::NeverRender,
				ColliderGizmos::AroundPlayer => {
					let collider_translation = collider_transform.translation();
					let debug = agents.iter().any(|agent_transform| {
						(agent_transform.translation() - collider_translation).length() < 10.
					});
					match debug {
						true => ColliderDebug::AlwaysRender,
						false => ColliderDebug::NeverRender,
					}
				}
				ColliderGizmos::All => ColliderDebug::AlwaysRender,
			};

			if current_debug == Some(&debug) {
				continue;
			};

			commands.try_apply_on(&entity, |mut e| {
				e.try_insert(debug);
			});
		}
	}
}

impl<TInput> Plugin for Debug<TInput>
where
	TInput: ThreadSafe + HandlesInput + SystemSetDefinition,
{
	fn build(&self, app: &mut App) {
		app.add_plugins(RapierDebugRenderPlugin::default())
			.init_resource::<ColliderGizmos>()
			.add_systems(
				Update,
				(
					Self::display_events,
					(Self::set_collider_gizmos, Self::control_collider_debug)
						.chain()
						.after_plugin(TInput::SYSTEMS),
				),
			);
	}
}

#[derive(Resource, Default)]
enum ColliderGizmos {
	#[default]
	None,
	AroundPlayer,
	All,
}

impl ColliderGizmos {
	fn rotate(&mut self) {
		*self = match self {
			Self::None => Self::AroundPlayer,
			Self::AroundPlayer => Self::All,
			Self::All => Self::None,
		}
	}
}
