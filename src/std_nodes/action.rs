//! Nodes that run a task in a separate thread.
use std::thread;
use std::sync::Arc;
use std::sync::mpsc;
use node::{Node, Internals};
use status::Status;

/// Represents the status of a taks
enum TaskStatus
{
	/// A task that has not been started yet
	Waiting,

	/// A task that is currently running
	Running(mpsc::Receiver<bool>),

	/// A task that has been completed
	Done(Status),
}

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

	/// Status of the task
	status: TaskStatus,
}
impl Action
{
	/// Creates a new Action node that will execute the given task.
	pub fn new<F>(task: F) -> Node
		where F: Fn() -> bool + Send + Sync + 'static
	{
		let internals = Action {
			func: Arc::new(task),
			status: TaskStatus::Waiting,
		};

		Node::new(internals)
	}

	/// Launches a new worker thread to run the task
	fn start_thread(&mut self) -> Status
	{
		// Create our new channels
		let (tx, rx) = mpsc::channel();

		// Then clone the function so we can move it
		let func_clone = self.func.clone();

		// Finally, boot up the thread
		thread::spawn(move || tx.send((func_clone)()).unwrap() );

		// Store the rx for later use
		self.status = TaskStatus::Running(rx);
		Status::Running
	}

	/// Checks if the worker thread is done or not
	fn check_thread(&mut self) -> Status
	{
		use std::sync::mpsc::TryRecvError;

		// Check on the status of the task
		let status = if let TaskStatus::Running(ref mut rx) = self.status {
			// See if there's anything waiting
			match rx.try_recv() {
				// Task was done, figure out the result
				Ok(res) => {
					if res { Status::Succeeded } else { Status::Failed }
				},

				// Still waiting on the task
				Err(TryRecvError::Empty) => Status::Running,

				// Something bad happend. Task died before finishing
				_ => panic!("Task died before finishing"),
			}
		} else { unreachable!() };

		// If we're done, we need move the task to the next stage
		self.status = TaskStatus::Done(status);
		status
	}
}
impl Internals for Action
{
	/// Returns `Status::Running` if the task has been started but not
	/// completed. Otherwise, it will return `Status::Succeeded` or
	/// `Status::Failed` based on the return value of the task.
	fn tick(&mut self) -> Status
	{
		match self.status {
			TaskStatus::Waiting      => self.start_thread(),
			TaskStatus::Running(_)   => self.check_thread(),
			TaskStatus::Done(status) => status
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
		if let TaskStatus::Running(ref mut rx) = self.status {
			rx.recv().unwrap();
		}
		self.status = TaskStatus::Waiting;
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
