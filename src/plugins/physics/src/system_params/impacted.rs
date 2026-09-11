pub mod iter_impacts;

use crate::components::impacted::Impacted;
use bevy::{
	ecs::system::{SystemParam, SystemParamItem},
	prelude::*,
};
use common::{prelude::*, traits::handles_physics::Impacted as ImpactedKey};

#[derive(SystemParam)]
pub struct ImpactedParam<'w, 's> {
	impacted: Query<'w, 's, Ref<'static, Impacted>>,
}

impl TryGetContext<ImpactedKey> for ImpactedParam<'static, 'static> {
	type TContext<'ctx> = ImpactedContext<'ctx>;

	fn try_get_context<'ctx>(
		param: &'ctx SystemParamItem<Self>,
		ImpactedKey { entity }: ImpactedKey,
	) -> Option<Self::TContext<'ctx>> {
		let impacted = param.impacted.get(entity).ok()?;

		Some(ImpactedContext { impacted })
	}
}

pub struct ImpactedContext<'ctx> {
	impacted: Ref<'ctx, Impacted>,
}

impl ContextChanged for ImpactedContext<'_> {
	fn context_changed(&self) -> bool {
		self.impacted.is_changed()
	}
}
