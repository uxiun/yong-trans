// #[cfg(debug_assertoins)]


// #[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! debug {
	($x:expr) => {
		std::convert::identity($x)
	};
}
