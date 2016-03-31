//#![crate_name="celix_rust"]
//#![feature(libc)]
#![allow(non_camel_case_types,non_snake_case)]

//! A bundle to test Celix and Rust interaction.
//! 
//! TODO: Allow more Rust like usage, preferably through use as (more optimised) library.

extern crate libc;
use libc::{c_void,malloc};

use std::fmt;
use std::sync::mpsc::Sender;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::mem::size_of;
use std::default::Default;

extern crate celix;
// #[macro_use]
// mod celix;
use celix::{celix_status_t,bundle_context_pt,CELIX_SUCCESS};
//const CELIX_SUCCESS: celix_status_t = 0;

// manifest!(SYMBOLIC_NAME = rust_celix);

/// Default thread count
const DEFAULT_THREADS: u64 = 4;

/// Data to be sent between Celix and this code.
/// 
/// `tx` is used to control the main worker thread: by sending `()`, the thread will stop.
#[repr(C)]
pub struct uData { 
	t1: i32,
	t2: i32,
	/// A `Sender` for closing the worker thread (cannot drop a dereference of a raw pointer safely!)
	tx: Sender<()>,
	work: *const Fn() -> i32
}

impl uData {
	pub fn work() {
		println!("Worker function called");
	}
}

impl Default for uData {
	fn default() -> uData {
		let (tmptx,_) = mpsc::channel();
		uData {
			t1:0,
			t2:0,
			tx:tmptx,
			work:&(||{0})
		}
	}
}

impl fmt::Debug for uData {
	/// Allows formatting
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "
uData {{ 
	t1: {:?}, 
	t2: {:?} 
}}", self.t1, self.t2)
	}
}

fn blah() -> i32 {
	println!("Working..."); 
	0
}

/// Create this bundle
#[no_mangle]
#[allow(unused_variables)]
pub extern "C" fn bundleActivator_create(context_ptr: bundle_context_pt, userData: *mut *mut c_void) -> celix_status_t {
	println!("create rust");
	// println!("{}", SYMBOLIC_NAME);
	unsafe {
		*userData = malloc(size_of::<uData>()); // C style malloc because the function receives a pointer
		(*(*userData as *mut uData)).t1 = 12;
		(*(*userData as *mut uData)).t2 = 100;
		(*(*userData as *mut uData)).work = & (|| { println!("Working..."); 0 });
		(*(*userData as *mut uData)).work = &blah; // Both approaches work
	}
	CELIX_SUCCESS
}

// pub extern "C" fn bundleActivator_create(context_ptr: bundle_context_pt, userData: *mut uData) -> celix_status_t {
// 	println!("create rust");
// 	let d = uData {
// 		t1: 12,
// 		t2: 100,
// 		work: &blah,
// 		..Default::default()
// 	};
// 	unsafe { (*userData) = d; }
// 	CELIX_SUCCESS
// }

/// Start this bundle.
///
/// Create the main worker thread, and let that create it's own child threads.
#[no_mangle]
#[allow(unused_variables)]
pub extern "C" fn bundleActivator_start(userData: *mut c_void, context: bundle_context_pt) -> celix_status_t {
	println!("start rust");
	let d : *mut uData;
	d = userData as *mut uData;
	println!("casted d");
	unsafe {
		println!("{:?}", (*d));
		let (tx, rx) = mpsc::channel();
		println!("created channel");
		(*d).tx = tx; // This seems to cause a SIGTRAP without compiler optimisations, will eventually have to find out why
		(*(*d).work)();
		thread::spawn(move ||{
			println!("started thread");
			// Aditional threads should be created here to allow proper control over them.
			let (ltx,lrx) = mpsc::channel();
			let mut threads = vec![];
			let NTHREADS: u64 = match std::env::var("RUST_THREADS") {
				Ok(s) => match s.parse::<u64>() {
					Ok(i) => i,
					Err(_) => DEFAULT_THREADS,
				},
				Err(_) => DEFAULT_THREADS,
			};

			for i in 0..NTHREADS { // in a..b, b is exclusive, so 0..2 == [0,1] 
				let ltx = ltx.clone();
				threads.push(thread::spawn(move || {
					loop {
						match ltx.send(()) {
							Err(SendError) => { // lrx has been dropped, so time to stop this thread
								println!("Stopping worker thread {}", i);
								return;
							},
							Ok(_) => {
								println!("worker thread {} working...", i);
								thread::sleep(Duration::new(0,100));
							},
						}
					}
				}));
			}
			loop {
				match rx.try_recv() {
					Ok(_)|Err(mpsc::TryRecvError::Disconnected) => {
						drop(lrx);
						for t in threads {
							let _ = t.join();
							println!("thread stopped");
						}
						println!("Terminated main worker thread");
						return;
					},
					Err(mpsc::TryRecvError::Empty) => {
						// println!("Working...");
						// thread::sleep(Duration::new(1,0));
					},
				}
			}
		});
	}
	println!("Hello {}", unsafe{(*d).t1});
	CELIX_SUCCESS
}

/// Stop this bundle
#[no_mangle]
#[allow(unused_variables)]
pub extern "C" fn bundleActivator_stop(userData: *mut c_void, context: bundle_context_pt) -> celix_status_t {
	println!("stop rust");
	let d : *const uData;
	d = userData as *const uData;
	unsafe { 
		println!("{:?}", (*d));
		(*(*d).work)();
		match (*d).tx.send(()) {
			Ok(_) => println!("Sent kill signal to worker thread..."),
			Err(_) => println!("Failed to send kill signal to worker thread..."),
		}
	}
	println!("Goodbye {}", unsafe{(*d).t1});
	CELIX_SUCCESS
}

/// Destroy this bundle
#[no_mangle]
#[allow(unused_variables)]
pub extern "C" fn bundleActivator_destroy(userData: *mut c_void, context: bundle_context_pt) -> celix_status_t {
	println!("destroy rust");
	CELIX_SUCCESS
}
