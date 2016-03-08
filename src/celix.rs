use libc::c_void;

/// Celix status code.
pub type celix_status_t = isize;
/// Successfully executed function.
pub const CELIX_SUCCESS: celix_status_t = 0;
/// Failed to start the bundle.
pub const CELIX_START_ERROR: celix_status_t = 70000;
/// Pointer to a context.
pub type bundle_context_pt = *mut c_void;

#[macro_export]
macro_rules! manifest {
	($key:ident = $val:expr, $($cont:expr),+) => (
		pub static $key: &'static str = stringify!($val);
		manifest!($($cont),+)
	);
	($key:ident = $val:expr) => (
		pub static $key: &'static str = stringify!($val);
		//println!("{} = {}", stringify!($key), stringify!($val));
	)
}