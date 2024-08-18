use std::marker::PhantomData;

use bevy::prelude::{Bundle, Component};

use super::{BlockedBy, ConcatBlockers};

pub(crate) struct Beam;

pub(crate) struct Fragile;

#[derive(Component)]
pub struct Is<T>(PhantomData<T>);

impl Is<()> {
	pub fn beam() -> Is<impl Sync + Send + 'static> {
		Is::<Beam>(PhantomData)
	}

	pub fn fragile() -> Is<impl Sync + Send + 'static> {
		Is::<Fragile>(PhantomData)
	}
}

impl<TIs> Is<TIs>
where
	Is<TIs>: Component,
{
	pub fn blocked_by<TBlockedBy: Component>(self) -> impl ConcatBlockers + Bundle {
		(self, BlockedBy::<TBlockedBy>(PhantomData))
	}
}

impl<TIs, TRest: Bundle> ConcatBlockers for (Is<TIs>, TRest)
where
	Is<TIs>: Component,
{
	fn and<TBlockedBy: Component>(self) -> impl ConcatBlockers + Bundle {
		let (is, blocker) = self;
		(is, (blocker, BlockedBy::<TBlockedBy>(PhantomData)))
	}
}
