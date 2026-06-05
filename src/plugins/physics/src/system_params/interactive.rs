mod iter_interactions;
mod iter_just_stopped;

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
	handles_physics::{InteractionsJustStopped, InteractionsOngoing},
};
use std::collections::HashSet;

#[derive(SystemParam, Debug)]
pub struct InteractiveParam<'w> {
	root_interactions: Res<'w, RootCollisions<Interactive>>,
}

impl GetContext<InteractionsOngoing> for InteractiveParam<'static> {
	type TContext<'ctx> = InteractiveContext<'ctx>;

	fn get_context<'ctx>(
		param: &'ctx SystemParamItem<Self>,
		InteractionsOngoing { entity }: InteractionsOngoing,
	) -> Self::TContext<'ctx> {
		InteractiveContext {
			changed: param.root_interactions.changed(&entity),
			interactions: param.root_interactions.ongoing(&entity),
		}
	}
}

impl GetContext<InteractionsJustStopped> for InteractiveParam<'static> {
	type TContext<'ctx> = JustStoppedInteractionsContext;

	fn get_context<'ctx>(
		param: &'ctx SystemParamItem<Self>,
		InteractionsJustStopped { entity }: InteractionsJustStopped,
	) -> Self::TContext<'ctx> {
		JustStoppedInteractionsContext {
			changed: param.root_interactions.changed(&entity),
			just_stopped: param.root_interactions.just_stopped(&entity),
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

pub struct JustStoppedInteractionsContext {
	changed: bool,
	just_stopped: HashSet<Entity>,
}

impl ContextChanged for JustStoppedInteractionsContext {
	fn context_changed(&self) -> bool {
		self.changed
	}
}
