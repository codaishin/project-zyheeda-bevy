use crate::{
	Tooltip,
	TooltipUIControl,
	components::tooltip::{TooltipUI, TooltipUiConfig},
	systems::{tooltip::tooltip, tooltip_visibility::tooltip_visibility},
};
use bevy::prelude::*;
use common::traits::handles_localization::LocalizeToken;

use super::insert_ui_content::InsertUiContent;

pub(crate) trait AddTooltip {
	fn add_tooltip<TLocalization, T>(&mut self) -> &mut Self
	where
		T: TooltipUiConfig + Clone + Sync + Send + 'static,
		Tooltip<T>: InsertUiContent,
		TLocalization: LocalizeToken + Resource;
}

impl AddTooltip for App {
	fn add_tooltip<TLocalization, T>(&mut self) -> &mut Self
	where
		T: TooltipUiConfig + Clone + Sync + Send + 'static,
		Tooltip<T>: InsertUiContent,
		TLocalization: LocalizeToken + Resource,
	{
		self.add_systems(
			Update,
			(
				tooltip::<T, TLocalization, TooltipUI<T>, TooltipUIControl, Window>,
				tooltip_visibility::<Real, T>,
			),
		)
	}
}
