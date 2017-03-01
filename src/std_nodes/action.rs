//! Nodes that run their a function in another thread
use std::thread;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use node::Node;
use status::Status;

/// Implements a standard action node.
pub struct Action<T: Send + Sync + 'static>
{
	/// Function that will be run to determine the status
	func: Arc<Fn(Arc<T>) -> bool + Send + Sync>,

	/// Handle to the thread running the function
	thread_handle: Option<thread::JoinHandle<Status>>,

	/// Flag that notifies this object that the work thread has completed
	flag: Arc<AtomicBool>,

	/// Status returned by the last tick
	status: Status,
}
impl<T: Send + Sync + 'static> Action<T>
{
	/// Launches a new worker thread to run the function
	fn start_thread(&mut self, world: &Arc<T>)
	{
		// Make sure our flag is set to false
		self.flag.store(false, Ordering::SeqCst);

		// Then boot up the thread
		let flag_clone = self.flag.clone();
		let func_clone = self.func.clone();
		let world_clone = world.clone();
		let thread_handle = thread::spawn(move || {
			// Run the function
			let res = (func_clone)(world_clone);

			// Set the flag so the main thread knows we're done
			flag_clone.store(true, Ordering::SeqCst);

			// Return the status
			if res { Status::Succeeded } else { Status::Failed }
		});

		// Store the handle for later
		self.thread_handle = Some(thread_handle);
	}
}
impl<T: Send + Sync + 'static> Node<T> for Action<T>
{
	fn tick(&mut self, world: &Arc<T>) -> Status
	{
		// First, check to see if we've already ran
		if self.status != Status::Running {
			return self.status;
		}

		// We haven't already run, so start up a new thread if needed
		if self.thread_handle.is_none() {
			self.start_thread(world);
		}

		// There is a thread running - get its status
		self.status = if !self.flag.load(Ordering::SeqCst) {
			Status::Running
		} else {
			// The thread is done, so load up its status. We also know that
			// we have a thread handle at this point
			let handle = self.thread_handle.take();
			handle.unwrap().join().unwrap()
		};

		// Return our status
		self.status
	}

	fn reset(&mut self)
	{
		// I debated what to do here for a while. I could see someone wanting to detach
		// the thread due to time constraints, but it seems to me that it would be better
		// to avoid potential bugs that come from a node only looking like its been
		// fully reset.
		self.status = Status::Running;
		self.flag.store(false, Ordering::SeqCst);
		if self.thread_handle.is_some() {
			let handle = self.thread_handle.take();
			handle.unwrap().join().unwrap();
		}
	}

	fn status(&self) -> Status
	{
		self.status
	}
}
