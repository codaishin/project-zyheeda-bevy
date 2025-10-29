use bevy::prelude::*;
use common::{
	tools::{Units, UnitsPerSecond, collider_radius::ColliderRadius, speed::Speed},
	traits::accessors::get::{DynProperty, GetProperty, TryApplyOn},
	zyheeda_commands::ZyheedaCommands,
};

#[derive(Component, Debug, PartialEq, Default)]
pub struct MovementDefinition {
	pub(crate) radius: Units,
	pub(crate) speed: UnitsPerSecond,
}

impl MovementDefinition {
	// FIXME: REMOVE THIS WHEN NOT NEEDED ANY MORE (SEE DOC)
	/// A temporary system until agents properly depend on this plugin
	/// and [`MovementDefinition`] can be inserted by other plugins.
	///
	/// Until then, use this to insert [`MovementDefinition`] from agents
	pub(crate) fn insert_from<TAgent>(
		mut commands: ZyheedaCommands,
		agents: Query<(Entity, &TAgent), Changed<TAgent>>,
	) where
		TAgent: Component + GetProperty<ColliderRadius> + GetProperty<Speed>,
	{
		for (entity, agent) in &agents {
			commands.try_apply_on(&entity, |mut e| {
				e.try_insert(Self {
					radius: agent.dyn_property::<ColliderRadius>(),
					speed: agent.dyn_property::<Speed>(),
				});
			});
		}
	}
}
