use bevy::prelude::Component;

#[derive(Component)]
pub struct Mark<T>(pub T);
