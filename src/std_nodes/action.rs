//! Nodes that run a task in a separate thread.
use std::thread;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use node::{Node, Internals};
use status::Status;

/// Implements a node that manages the execution of tasks.
///
/// This node has no children but instead accepts a function that will be used
/// to evaluate the status of the node. This function will be ran in a separate
/// thread with the return value of the function specifying whether it succeeded
/// or failed. This node will be considered `Running` as long as the function is
/// running.
pub struct Action
{
	/// The task which is to be run
	func: Arc<Fn() -> bool + Send + Sync>,

	/// Handle to the thread running the task
	thread_handle: Option<thread::JoinHandle<Status>>,

	/// Value that the thread returned
	thread_res: Option<Status>,

	/// Flag that notifies this object that the work thread has completed
	flag: Arc<AtomicBool>,
}
impl Action
{
	/// Creates a new Action node that will execute the given task.
	pub fn new<F>(task: F) -> Node
		where F: Fn() -> bool + Send + Sync + 'static
	{
		let internals = Action {
			func: Arc::new(task),
			thread_handle: None,
			thread_res: None,
			flag: Arc::new(AtomicBool::new(false)),
		};

		Node::new(internals)
	}

	/// Launches a new worker thread to run the task
	fn start_thread(&mut self)
	{
		// Make sure our flag is set to false
		self.flag.store(false, Ordering::SeqCst);

		// Then boot up the thread
		let flag_clone = self.flag.clone();
		let func_clone = self.func.clone();
		let thread_handle = thread::spawn(move || {
			// Run the function
			let res = (func_clone)();

			// Set the flag so the main thread knows we're done
			flag_clone.store(true, Ordering::SeqCst);

			// Return the status
			if res { Status::Succeeded } else { Status::Failed }
		});

		// Store the handle for later
		self.thread_handle = Some(thread_handle);
	}
}
impl Internals for Action
{
	/// Returns `Status::Running` if the task has been started but not
	/// completed. Otherwise, it will return `Status::Succeeded` or
	/// `Status::Failed` based on the return value of the task.
	fn tick(&mut self) -> Status
	{
		// First, check to see if we've already ran
		if let Some(res) = self.thread_res {
			return res;
		}

		// We haven't already run, so start up a new thread if needed
		if self.thread_handle.is_none() {
			self.start_thread();
		}

		// There is a thread running - get its status
		if !self.flag.load(Ordering::SeqCst) {
			Status::Running
		} else {
			// The thread is done, so load up its status. We also know that
			// we have a thread handle at this point
			let handle = self.thread_handle.take();
			let status = handle.unwrap().join().unwrap();

			self.thread_res = Some(status);
			status
		}
	}

	/// Resets the node to a state identical to when it was first constructed.
	///
	/// If there is a running task, this function will block until the task is
	/// completed.
	fn reset(&mut self)
	{
		// I debated what to do here for a while. I could see someone wanting to detach
		// the thread due to time constraints, but it seems to me that it would be better
		// to avoid potential bugs that come from a node only looking like its been
		// fully reset.
		self.flag.store(false, Ordering::SeqCst);
		self.thread_res = None;
		if self.thread_handle.is_some() {
			let handle = self.thread_handle.take();
			handle.unwrap().join().unwrap();
		}
	}

	/// Returns the string "Action"
	fn type_name(&self) -> &'static str
	{
		"Action"
	}
}

#[cfg(test)]
mod test
{
	use std::sync::Mutex;
	use std::sync::mpsc;
	use std::sync::mpsc::{Sender, Receiver};
	use std::time;
	use std::thread;
	use status::Status;
	use std_nodes::*;

	#[test]
	fn failure()
	{
		let (tx, rx): (Sender<bool>, Receiver<bool>) = mpsc::channel();

		let mrx = Mutex::new(rx);
		let mut action = Action::new(move || {
			// Block until the message is sent, then return its value
			mrx.lock().unwrap().recv().unwrap()
		});

		for _ in 0..5 {
			assert_eq!(action.tick(), Status::Running);
			thread::sleep(time::Duration::from_millis(100));
		}

		tx.send(false).unwrap();

		let mut status = Status::Running;
		while status == Status::Running {
			status = action.tick();
		}

		assert_eq!(status, Status::Failed);
	}

	#[test]
	fn success()
	{
		let (tx, rx): (Sender<bool>, Receiver<bool>) = mpsc::channel();

		let mrx = Mutex::new(rx);
		let mut action = Action::new(move || {
			// Block until the message is sent, then return its value
			mrx.lock().unwrap().recv().unwrap()
		});

		for _ in 0..5 {
			assert_eq!(action.tick(), Status::Running);
			thread::sleep(time::Duration::from_millis(100));
		}

		tx.send(true).unwrap();

		let mut status = Status::Running;
		while status == Status::Running {
			status = action.tick();
		}

		assert_eq!(status, Status::Succeeded);
	}
}
