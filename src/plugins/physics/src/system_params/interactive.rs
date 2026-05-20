mod overlaps_with;

use crate::{
	components::physical_body::Interactive,
	resources::ongoing_interactions::OngoingInteractions,
};
use bevy::{
	ecs::system::{SystemParam, SystemParamItem},
	prelude::*,
};
use common::traits::{
	accessors::get::{ContextChanged, GetContext},
	handles_physics::HasInteractiveFrame,
};
use std::collections::HashSet;

#[derive(SystemParam, Debug)]
pub struct InteractiveParam<'w> {
	child_colliders: Res<'w, OngoingInteractions<Interactive>>,
}

impl GetContext<HasInteractiveFrame> for InteractiveParam<'static> {
	type TContext<'ctx> = InteractiveContext<'ctx>;

	fn get_context<'ctx>(
		param: &'ctx SystemParamItem<Self>,
		HasInteractiveFrame { entity }: HasInteractiveFrame,
	) -> Option<Self::TContext<'ctx>> {
		Some(InteractiveContext {
			interactions: param.child_colliders.interactions.get(&entity)?,
		})
	}
}

pub struct InteractiveContext<'ctx> {
	interactions: &'ctx HashSet<Entity>,
}

impl ContextChanged for InteractiveContext<'_> {
	fn context_changed(&self) -> bool {
		true
	}
}
