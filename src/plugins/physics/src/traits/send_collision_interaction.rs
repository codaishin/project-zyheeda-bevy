use bevy::prelude::*;

pub(crate) trait PushOngoingInteraction {
	fn push_ongoing_interaction(&mut self, actor: Entity, target: Entity);
}
