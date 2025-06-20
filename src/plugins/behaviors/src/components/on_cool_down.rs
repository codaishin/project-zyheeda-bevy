pub(crate) mod dto;

use crate::components::on_cool_down::dto::OnCoolDownDto;
use bevy::prelude::*;
use common::traits::handles_saving::SavableComponent;
use std::time::Duration;

#[derive(Component, Debug, PartialEq, Clone)]
pub(crate) struct OnCoolDown(pub Duration);

impl SavableComponent for OnCoolDown {
	type TDto = OnCoolDownDto;
}
