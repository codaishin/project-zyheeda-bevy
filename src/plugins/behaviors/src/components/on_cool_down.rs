pub(crate) mod dto;

use crate::components::on_cool_down::dto::OnCoolDownDto;
use bevy::prelude::*;
use macros::SavableComponent;
use std::time::Duration;

#[derive(Component, SavableComponent, Debug, PartialEq, Clone)]
#[savable_component(dto = OnCoolDownDto)]
pub(crate) struct OnCoolDown(pub Duration);
