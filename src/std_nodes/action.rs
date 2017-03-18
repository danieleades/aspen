//! Nodes that run their a function in another thread
use std::thread;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use node::{Node, Internals};
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
}
impl<T: Send + Sync + 'static> Action<T>
{
	/// Creates a new Action node
	pub fn new(func: Arc<Fn(Arc<T>) -> bool + Send + Sync>) -> Node<T>
	{
		let internals = Action {
			func: func,
			thread_handle: None,
			flag: Arc::new(AtomicBool::new(false)),
		};

		Node::new(internals)
	}

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
impl<T: Send + Sync + 'static> Internals<T> for Action<T>
{
	fn tick(&mut self, world: &Arc<T>) -> Status
	{
		// First, check to see if we've already ran
		if self.status().is_done() {
			return self.status();
		}

		// We haven't already run, so start up a new thread if needed
		if self.thread_handle.is_none() {
			assert_eq!(self.status(), Status::Initialized);
			self.start_thread(world);
		}

		// There is a thread running - get its status
		if !self.flag.load(Ordering::SeqCst) {
			Status::Running
		} else {
			// The thread is done, so load up its status. We also know that
			// we have a thread handle at this point
			let handle = self.thread_handle.take();
			handle.unwrap().join().unwrap()
		}
	}

	fn reset(&mut self)
	{
		// I debated what to do here for a while. I could see someone wanting to detach
		// the thread due to time constraints, but it seems to me that it would be better
		// to avoid potential bugs that come from a node only looking like its been
		// fully reset.
		self.flag.store(false, Ordering::SeqCst);
		if self.thread_handle.is_some() {
			let handle = self.thread_handle.take();
			handle.unwrap().join().unwrap();
		}
	}

	fn type_name(&self) -> &'static str
	{
		"Action"
	}
}

#[cfg(test)]
mod test
{
	use std::sync::Arc;
	use std::sync::atomic::{AtomicUsize, Ordering};
	use std::{thread, time};
	use node::Node;
	use status::Status;
	use std_nodes::*;

	fn action_func(world: Arc<AtomicUsize>) -> bool
	{
		while world.load(Ordering::SeqCst) == 0 {
			thread::yield_now();
		}

		world.load(Ordering::SeqCst) == 1
	}

	#[test]
	fn failure()
	{
		let world = Arc::new(AtomicUsize::new(0));
		let mut action = Action::new(Arc::new(action_func));

		for _ in 0..5 {
			assert_eq!(action.tick(&world), Status::Running);
			thread::sleep(time::Duration::from_millis(100));
		}

		world.store(2, Ordering::SeqCst);

		let mut status = Status::Running;
		while status == Status::Running {
			status = action.tick(&world);
		}

		assert_eq!(status, Status::Failed);
	}

	#[test]
	fn success()
	{
		let world = Arc::new(AtomicUsize::new(0));
		let mut action = Action::new(Arc::new(action_func));

		for _ in 0..5 {
			assert_eq!(action.tick(&world), Status::Running);
			thread::sleep(time::Duration::from_millis(100));
		}

		world.store(1, Ordering::SeqCst);

		let mut status = Status::Running;
		while status == Status::Running {
			status = action.tick(&world);
		}

		assert_eq!(status, Status::Succeeded);
	}
}
