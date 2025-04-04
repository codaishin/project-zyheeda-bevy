use crate::traits::iteration::{Iter, IterFinite};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum AnimationKey<TOther> {
	T,
	Idle,
	Walk,
	Run,
	Other(TOther),
}

impl<TOther> IterFinite for AnimationKey<TOther>
where
	TOther: Copy + IterFinite,
{
	fn iterator() -> Iter<Self> {
		Iter(Some(AnimationKey::T))
	}

	fn next(current: &Iter<Self>) -> Option<Self> {
		let Iter(current) = current;
		match current.as_ref()? {
			AnimationKey::T => Some(AnimationKey::Idle),
			AnimationKey::Idle => Some(AnimationKey::Walk),
			AnimationKey::Walk => Some(AnimationKey::Run),
			AnimationKey::Run => Some(AnimationKey::Other(TOther::iterator().0?)),
			AnimationKey::Other(other) => {
				TOther::next(&Iter(Some(*other))).map(AnimationKey::Other)
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Debug, PartialEq, Clone, Copy)]
	struct _Other;

	impl IterFinite for _Other {
		fn iterator() -> Iter<Self> {
			Iter(Some(Self))
		}

		fn next(current: &Iter<Self>) -> Option<Self> {
			let Iter(current) = current;
			match current.as_ref()? {
				Self => None,
			}
		}
	}

	#[test]
	fn iterate() {
		assert_eq!(
			vec![
				AnimationKey::T,
				AnimationKey::Idle,
				AnimationKey::Walk,
				AnimationKey::Run,
				AnimationKey::Other(_Other),
			],
			AnimationKey::<_Other>::iterator()
				.take(10) // avoiding infinite hang when broken
				.collect::<Vec<_>>()
		)
	}
}
