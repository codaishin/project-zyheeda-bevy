use super::insert_ui_content::InsertUiContent;
use crate::components::tooltip::Tooltip;
use bevy::prelude::*;
use common::traits::{handles_localization::LocalizeToken, thread_safe::ThreadSafe};

pub(crate) trait DespawnAllTooltips<TUI> {
	fn despawn_all(
		&self,
		uis: &Query<(Entity, &TUI, &mut Node, &ComputedNode)>,
		commands: &mut Commands,
	) where
		TUI: Component + Sized;
}

pub(crate) trait DespawnOutdatedTooltips<TUI, T>
where
	T: ThreadSafe,
{
	fn despawn_outdated(
		&self,
		uis: &Query<(Entity, &TUI, &mut Node, &ComputedNode)>,
		commands: &mut Commands,
		outdated_tooltips: RemovedComponents<Tooltip<T>>,
	) where
		TUI: Component + Sized;
}

pub(crate) trait UpdateTooltipPosition<TUI> {
	fn update_position(
		&self,
		uis: &mut Query<(Entity, &TUI, &mut Node, &ComputedNode)>,
		position: MouseVec2,
	) where
		TUI: Component + Sized;
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct MouseVec2(pub(crate) Vec2);

pub(crate) trait SpawnTooltips<T, TLocalization>
where
	TLocalization: LocalizeToken + ThreadSafe,
{
	fn spawn(
		&self,
		commands: &mut Commands,
		localize: &mut TLocalization,
		tooltip_entity: Entity,
		tooltip: &Tooltip<T>,
	) where
		Tooltip<T>: InsertUiContent;
}
