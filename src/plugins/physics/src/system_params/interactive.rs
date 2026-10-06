mod iter_interactions;

use crate::{
	components::{collision_domains::Interactive, interactions_changed::InteractionsChanged},
	resources::root_collisions::RootCollisions,
};
use bevy::{
	ecs::system::{SystemParam, SystemParamItem},
	prelude::*,
};
use common::prelude::*;
use std::collections::HashSet;

#[derive(SystemParam, Debug)]
pub struct InteractiveParam<'w, 's> {
	root_interactions: Res<'w, RootCollisions<Interactive>>,
	markers: Query<'w, 's, Ref<'static, InteractionsChanged>>,
}

impl GetContext<InteractionsOngoing> for InteractiveParam<'static, 'static> {
	type TContext<'ctx> = InteractiveContext<'ctx>;

	fn get_context<'ctx>(
		param: &'ctx SystemParamItem<Self>,
		InteractionsOngoing { entity }: InteractionsOngoing,
	) -> Self::TContext<'ctx> {
		InteractiveContext {
			marker: param.markers.get(entity).ok(),
			interactions: param.root_interactions.ongoing(&entity),
		}
	}
}

pub struct InteractiveContext<'ctx> {
	marker: Option<Ref<'ctx, InteractionsChanged>>,
	interactions: &'ctx HashSet<Entity>,
}

impl ContextChanged for InteractiveContext<'_> {
	fn context_changed(&self) -> bool {
		self.marker.map(|m| m.is_changed()).unwrap_or_default()
	}
}
