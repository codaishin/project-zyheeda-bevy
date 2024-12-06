use super::responsive_light::ResponsiveLight;
use bevy::prelude::*;

#[derive(Component, Debug, PartialEq)]
pub(crate) enum ResponsiveLightChange {
	Increase(ResponsiveLight),
	Decrease(ResponsiveLight),
}
