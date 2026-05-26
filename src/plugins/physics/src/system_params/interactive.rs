mod iter_interactions;

use crate::{
	components::collision_domains::Interactive,
	resources::ongoing_interactions::OngoingInteractions,
};
use bevy::{
	ecs::system::{SystemParam, SystemParamItem},
	prelude::*,
};
use common::traits::{
	accessors::get::{ContextChanged, GetContext},
	handles_physics::Interactions,
};
use std::{collections::HashSet, sync::LazyLock};

#[derive(SystemParam, Debug)]
pub struct InteractiveParam<'w> {
	child_colliders: Res<'w, OngoingInteractions<Interactive>>,
}

static EMPTY: LazyLock<HashSet<Entity>> = LazyLock::new(HashSet::default);

impl GetContext<Interactions> for InteractiveParam<'static> {
	type TContext<'ctx> = InteractiveContext<'ctx>;

	fn get_context<'ctx>(
		param: &'ctx SystemParamItem<Self>,
		Interactions { entity }: Interactions,
	) -> Self::TContext<'ctx> {
		InteractiveContext {
			interactions: param
				.child_colliders
				.interactions
				.get(&entity)
				.unwrap_or(&*EMPTY),
		}
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
