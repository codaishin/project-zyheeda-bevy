use super::insert_ui_content::InsertUiContent;
use crate::components::tooltip::Tooltip;
use bevy::prelude::*;
use common::traits::{handles_localization::LocalizeToken, thread_safe::ThreadSafe};

pub(crate) trait DespawnAllTooltips<TUI> {
	fn despawn_all(&self, uis: &Query<(Entity, &TUI, &mut Node)>, commands: &mut Commands)
	where
		TUI: Component + Sized;
}

pub(crate) trait DespawnOutdatedTooltips<TUI, T>
where
	T: ThreadSafe,
{
	fn despawn_outdated(
		&self,
		uis: &Query<(Entity, &TUI, &mut Node)>,
		commands: &mut Commands,
		outdated_tooltips: RemovedComponents<Tooltip<T>>,
	) where
		TUI: Component + Sized;
}

pub(crate) trait UpdateTooltipPosition<TUI> {
	fn update_position(&self, uis: &mut Query<(Entity, &TUI, &mut Node)>, position: Vec2)
	where
		TUI: Component + Sized;
}

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
		position: Vec2,
	) where
		Tooltip<T>: InsertUiContent;
}
