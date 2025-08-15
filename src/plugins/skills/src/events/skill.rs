use bevy::prelude::*;
use common::tools::action_key::slot::SlotKey;

#[derive(Event, Debug, PartialEq, Clone, Copy)]
pub(crate) enum SkillEvent {
	Hold { agent: Entity, key: SlotKey },
	Release { agent: Entity, key: SlotKey },
}
