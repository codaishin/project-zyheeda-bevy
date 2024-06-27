use super::{get_node::GetNode, instantiate_content_on::InstantiateContentOn};
use crate::components::tooltip::Tooltip;
use bevy::{
	math::Vec2,
	prelude::{Commands, Component, Entity, Query, RemovedComponents},
	ui::Style,
};

pub(crate) trait DespawnAllTooltips<TUI> {
	fn despawn_all(&self, uis: &Query<(Entity, &TUI, &mut Style)>, commands: &mut Commands)
	where
		TUI: Component + Sized;
}

pub(crate) trait DespawnOutdatedTooltips<TUI, T: Send + Sync + 'static> {
	fn despawn_outdated(
		&self,
		uis: &Query<(Entity, &TUI, &mut Style)>,
		commands: &mut Commands,
		outdated_tooltips: RemovedComponents<Tooltip<T>>,
	) where
		TUI: Component + Sized;
}

pub(crate) trait UpdateTooltipPosition<TUI> {
	fn update_position(&self, uis: &mut Query<(Entity, &TUI, &mut Style)>, position: Vec2)
	where
		TUI: Component + Sized;
}

pub(crate) trait SpawnTooltips<T> {
	fn spawn(
		&self,
		commands: &mut Commands,
		tooltip_entity: Entity,
		tooltip: &Tooltip<T>,
		position: Vec2,
	) where
		Tooltip<T>: InstantiateContentOn + GetNode;
}
