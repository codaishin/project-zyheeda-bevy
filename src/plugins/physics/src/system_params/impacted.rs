pub mod iter_impacts;

use bevy::ecs::system::{SystemParam, SystemParamItem};
use common::{prelude::*, traits::handles_physics::Impacted as ImpactedKey};

#[derive(SystemParam)]
pub struct ImpactedParam {}

impl TryGetContext<ImpactedKey> for ImpactedParam {
	type TContext<'ctx> = ImpactedContext;

	fn try_get_context<'ctx>(
		_: &'ctx SystemParamItem<Self>,
		_: ImpactedKey,
	) -> Option<Self::TContext<'ctx>> {
		None
	}
}

pub struct ImpactedContext {}

impl ContextChanged for ImpactedContext {
	fn context_changed(&self) -> bool {
		false
	}
}
