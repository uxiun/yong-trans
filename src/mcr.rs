// #[cfg(debug_assertoins)]
#[macro_export]
macro_rules! d {
	($x:expr) => {
		dbg!(&$x)
	};
}

// #[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! debug {
	($x:expr) => {
		std::convert::identity($x)
	};
}
