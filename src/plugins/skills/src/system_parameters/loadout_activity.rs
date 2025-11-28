mod active_skills;

use crate::components::queue::Queue;
use bevy::{ecs::system::SystemParam, prelude::*};
use common::traits::{
	accessors::get::{ContextChanged, GetContext},
	handles_loadout::skills::Skills,
};

#[derive(SystemParam)]
pub struct LoadoutActivity<'w, 's> {
	queues: Query<'w, 's, Ref<'static, Queue>>,
}

impl GetContext<Skills> for LoadoutActivity<'_, '_> {
	type TContext<'ctx> = LoadoutActivityContext<'ctx>;

	fn get_context<'ctx>(
		param: &'ctx LoadoutActivity,
		Skills { entity }: Skills,
	) -> Option<Self::TContext<'ctx>> {
		let queue = param.queues.get(entity).ok()?;

		Some(LoadoutActivityContext { queue })
	}
}

pub struct LoadoutActivityContext<'a> {
	queue: Ref<'a, Queue>,
}

impl ContextChanged for LoadoutActivityContext<'_> {
	fn context_changed(&self) -> bool {
		self.queue.is_changed()
	}
}
