use bevy::prelude::Component;

#[derive(Component)]
pub(crate) struct Tooltip<T>(pub T);
