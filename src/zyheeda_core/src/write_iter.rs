#[macro_export]
macro_rules! write_iter {
	($fmt:expr, $iter:expr) => {{
		write!($fmt, "[")?;
		let mut iter = $iter.iter();
		if let Some(first) = iter.next() {
			write!($fmt, "{first}")?;
		}
		for item in iter {
			write!($fmt, "{item}")?;
		}
		write!($fmt, "]")
	}};
	($fmt:expr, $label:literal, $iter:expr) => {{
		write!($fmt, $label)?;
		write_iter!($fmt, $iter)
	}};
}

pub use write_iter;
