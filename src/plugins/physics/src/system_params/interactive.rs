mod iter_interactions;

use crate::{
	components::collision_domains::Interactive,
	resources::root_collisions::RootCollisions,
};
use bevy::{
	ecs::system::{SystemParam, SystemParamItem},
	prelude::*,
};
use common::traits::{
	accessors::get::{ContextChanged, GetContext},
	handles_physics::Interactions,
};
use std::collections::HashSet;

#[derive(SystemParam, Debug)]
pub struct InteractiveParam<'w> {
	root_interactions: Res<'w, RootCollisions<Interactive>>,
}

impl GetContext<Interactions> for InteractiveParam<'static> {
	type TContext<'ctx> = InteractiveContext<'ctx>;

	fn get_context<'ctx>(
		param: &'ctx SystemParamItem<Self>,
		Interactions { entity }: Interactions,
	) -> Self::TContext<'ctx> {
		InteractiveContext {
			changed: param.root_interactions.changed(&entity),
			interactions: param.root_interactions.ongoing(&entity),
		}
	}
}

pub struct InteractiveContext<'ctx> {
	changed: bool,
	interactions: &'ctx HashSet<Entity>,
}

impl ContextChanged for InteractiveContext<'_> {
	fn context_changed(&self) -> bool {
		self.changed
	}
}
