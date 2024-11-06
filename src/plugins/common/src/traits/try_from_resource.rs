use bevy::prelude::Resource;

pub trait TryFromResource<TFrom>
where
	Self: Sized,
{
	type TResource: Resource;
	type TError;

	fn try_from_resource(from: TFrom, resource: &Self::TResource) -> Result<Self, Self::TError>;
}
