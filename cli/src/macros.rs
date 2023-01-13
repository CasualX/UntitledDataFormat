
macro_rules! expect {
	($e:expr, $($tt:tt)*) => {
		$crate::macros::Presume::presume($e, ::fmtools::fmt!($($tt)*))
	};
}

macro_rules! error {
	($($tt:tt)*) => {
		$crate::macros::exit_error(None, &::fmtools::fmt!($($tt)*))
	};
}

#[track_caller]
#[inline(never)]
pub fn exit_error(err: Option<&dyn std::error::Error>, msg: &dyn std::fmt::Display) -> ! {
	if let Some(err) = err {
		eprintln!("{}\nerror: {}\n   --> {}", msg, err, std::panic::Location::caller());
	}
	else {
		eprintln!("{}\n   --> {}", msg, std::panic::Location::caller());
	}
	std::process::exit(1)
}

pub trait Presume<T> {
	#[track_caller]
	fn presume(self, msg: impl std::fmt::Display) -> T;
}

impl<T, E: std::error::Error> Presume<T> for Result<T, E> {
	#[track_caller]
	#[inline]
	fn presume(self, msg: impl std::fmt::Display) -> T {
		match self {
			Ok(v) => v,
			Err(err) => exit_error(Some(&err), &msg),
		}
	}
}

impl<T> Presume<T> for Option<T> {
	#[track_caller]
	#[inline]
	fn presume(self, msg: impl std::fmt::Display) -> T {
		match self {
			Some(v) => v,
			None => exit_error(None, &msg),
		}
	}
}
