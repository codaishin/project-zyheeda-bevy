use crate::errors::ErrorData;
use bevy::{ecs::system::SystemParam, prelude::*};
use std::{
	any::TypeId,
	collections::{HashMap, hash_map::Entry},
	ops::Deref,
	sync::{LazyLock, RwLock, RwLockReadGuard, RwLockWriteGuard},
	time::{Duration, Instant},
};

pub trait Log {
	fn log<TError>(&self, error: TError)
	where
		TError: ErrorData;
}

impl<T> Log for T
where
	T: Deref<Target: Log>,
{
	fn log<TError>(&self, error: TError)
	where
		TError: ErrorData,
	{
		self.deref().log(error);
	}
}

/// A global error logger system parameter.
///
/// Each instance of this will use the same internal error logger singleton for logging.
#[derive(SystemParam)]
pub struct GlobalErrorLogger;

impl GlobalErrorLogger {
	/// The global error logger instance.
	///
	/// For testability it is recommended to test against generic system parameters
	/// implementing [`Log`] and then injecting the [`GlobalErrorLogger`] type as a system parameter
	/// dependency.
	pub const INSTANCE: Self = Self;

	const LOCK_FAILED: &str = "Cannot obtain global error logger instance";

	fn lock_write() -> RwLockWriteGuard<'static, ErrorLogger> {
		// SAFETY: This only fails if another user panicked while writing, at which point the
		//         app would have already crashed anyway
		#[allow(clippy::expect_used)]
		ERROR_LOGGER.write().expect(Self::LOCK_FAILED)
	}

	fn lock_read() -> RwLockReadGuard<'static, ErrorLogger> {
		// SAFETY: This only fails if another user panicked while writing, at which point the
		//         app would have already crashed anyway
		#[allow(clippy::expect_used)]
		ERROR_LOGGER.read().expect(Self::LOCK_FAILED)
	}

	pub(crate) fn remove_elapsed() {
		if Self::lock_read().suppress.is_empty() {
			return;
		}

		Self::lock_write().remove_elapsed();
	}
}

impl Log for GlobalErrorLogger {
	fn log<TError>(&self, error: TError)
	where
		TError: ErrorData,
	{
		let mut logger = Self::lock_write();

		logger.log(error);
	}
}

static ERROR_LOGGER: LazyLock<RwLock<ErrorLogger>> = LazyLock::new(RwLock::default);

#[derive(Default)]
struct ErrorLogger {
	suppress: HashMap<TypeId, (Instant, Duration)>,
	#[cfg(test)]
	entries: Vec<tests::LogEntry>,
}

impl ErrorLogger {
	fn log<TError>(&mut self, error: TError)
	where
		TError: ErrorData,
	{
		if self.is_suppressed::<TError>() {
			return;
		}

		self.write(error);
	}

	fn remove_elapsed(&mut self) {
		let elapsed = self
			.suppress
			.iter()
			.filter_map(|(e, (last_log, limit))| match (last_log.elapsed(), limit) {
				(elapsed, limit) if elapsed > *limit => Some(*e),
				_ => None,
			})
			.collect::<Vec<_>>();

		for elapsed in elapsed {
			self.suppress.remove(&elapsed);
		}
	}

	fn is_suppressed<TError>(&mut self) -> bool
	where
		TError: ErrorData + 'static,
	{
		let Some(limit) = TError::rate_limit() else {
			return false;
		};

		match self.suppress.entry(TypeId::of::<TError>()) {
			Entry::Occupied(e) if e.get().0.elapsed() < limit => return true,
			Entry::Occupied(mut e) => {
				e.insert((Instant::now(), limit));
			}
			Entry::Vacant(e) => {
				e.insert((Instant::now(), limit));
			}
		}

		false
	}

	fn write<TError>(&mut self, error: TError)
	where
		TError: ErrorData,
	{
		let level = error.level();
		let label = TError::label().to_string();

		#[cfg(test)]
		{
			let details = error.into_details().to_string();

			self.entries.push(tests::LogEntry {
				level,
				label,
				details,
			});
		}

		#[cfg(not(test))]
		{
			use crate::errors::Level;
			use tracing::{error, field::display, warn};

			let details = display(error.into_details());

			match level {
				Level::Error => {
					error!(details, "{label}");
				}
				Level::Warning => {
					warn!(details, "{label}");
				}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::errors::Level;
	use std::{thread, time::Duration};

	#[derive(Debug, PartialEq, Clone)]
	pub(super) struct LogEntry {
		pub(super) level: Level,
		pub(super) label: String,
		pub(super) details: String,
	}

	struct _Error;

	impl ErrorData for _Error {
		fn level(&self) -> Level {
			Level::Error
		}

		fn label() -> impl std::fmt::Display {
			"ERROR"
		}

		fn into_details(self) -> impl std::fmt::Display {
			"DETAILS"
		}
	}

	struct _LimitedError;

	impl ErrorData for _LimitedError {
		fn rate_limit() -> Option<Duration> {
			Some(Duration::from_millis(500))
		}

		fn level(&self) -> Level {
			Level::Error
		}

		fn label() -> impl std::fmt::Display {
			"LIMITED ERROR"
		}

		fn into_details(self) -> impl std::fmt::Display {
			"DETAILS"
		}
	}

	#[test]
	fn log_error() {
		let mut logger = ErrorLogger::default();

		logger.log(_Error);

		assert_eq!(
			vec![LogEntry {
				level: Level::Error,
				label: String::from("ERROR"),
				details: String::from("DETAILS")
			}],
			logger.entries,
		);
	}

	#[test]
	fn limit_rate() {
		let mut logger = ErrorLogger::default();

		logger.log(_LimitedError);
		logger.log(_LimitedError);

		assert_eq!(
			vec![LogEntry {
				level: Level::Error,
				label: String::from("LIMITED ERROR"),
				details: String::from("DETAILS")
			}],
			logger.entries,
		);
	}

	#[test]
	fn log_again_after_rate_limit_expired() {
		let mut logger = ErrorLogger::default();

		logger.log(_LimitedError);
		thread::sleep(Duration::from_millis(750));
		logger.log(_LimitedError);

		assert_eq!(
			vec![
				LogEntry {
					level: Level::Error,
					label: String::from("LIMITED ERROR"),
					details: String::from("DETAILS")
				},
				LogEntry {
					level: Level::Error,
					label: String::from("LIMITED ERROR"),
					details: String::from("DETAILS")
				},
			],
			logger.entries,
		);
	}

	#[test]
	fn log_again_when_no_rate_limit() {
		let mut logger = ErrorLogger::default();

		logger.log(_Error);
		logger.log(_Error);

		assert_eq!(
			vec![
				LogEntry {
					level: Level::Error,
					label: String::from("ERROR"),
					details: String::from("DETAILS")
				},
				LogEntry {
					level: Level::Error,
					label: String::from("ERROR"),
					details: String::from("DETAILS")
				},
			],
			logger.entries,
		);
	}

	#[test]
	fn remove_elapsed_errors() {
		let mut logger = ErrorLogger::default();

		logger.log(_LimitedError);
		thread::sleep(Duration::from_millis(750));
		logger.remove_elapsed();

		assert!(!logger.suppress.contains_key(&TypeId::of::<_LimitedError>()));
	}

	#[test]
	fn do_not_remove_no_elapsed() {
		let mut logger = ErrorLogger::default();

		logger.log(_LimitedError);
		logger.remove_elapsed();

		assert!(logger.suppress.contains_key(&TypeId::of::<_LimitedError>()));
	}
}
