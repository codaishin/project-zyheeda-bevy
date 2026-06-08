use super::insert_ui_content::InsertUiContent;
use crate::{
	Tooltip,
	TooltipUIControl,
	components::tooltip::{TooltipContent, TooltipUiConfig},
	systems::{tooltip::tooltip, tooltip_visibility::tooltip_visibility},
};
use bevy::{ecs::system::SystemParam, prelude::*};
use common::traits::{handles_localization::Localize, thread_safe::ThreadSafe};

pub(crate) trait AddTooltip {
	fn add_tooltip<TLocalization, T>(&mut self) -> &mut Self
	where
		T: TooltipUiConfig + ThreadSafe,
		Tooltip<T>: InsertUiContent,
		TLocalization: for<'w, 's> SystemParam<Item<'w, 's>: Localize> + ThreadSafe;
}

impl AddTooltip for App {
	fn add_tooltip<TLocalization, T>(&mut self) -> &mut Self
	where
		T: TooltipUiConfig + ThreadSafe,
		Tooltip<T>: InsertUiContent,
		TLocalization: for<'w, 's> SystemParam<Item<'w, 's>: Localize> + ThreadSafe,
	{
		self.add_systems(
			Update,
			(
				tooltip::<T, TLocalization, TooltipContent<T>, TooltipUIControl, Window>,
				tooltip_visibility::<Real, T>,
			),
		)
	}
}
