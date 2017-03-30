//! Nodes that cause the execution of tasks.
use std::thread;
use std::sync::Arc;
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;
use node::{Node, Internals};
use status::Status;

/// A node that manages the execution of tasks in a separate thread.
///
/// This node will launch the supplied function in a separate thread and ticks
/// will monitor the state of that thread. If the supplied function returns
/// `true` then the node is considered successful, otherwise it is considered to
/// have failed.
///
/// This node should be the main way of modifying the world state. Note that
/// most, in most cases, there will only be one thread modifying the world.
///
/// # State
///
/// **Initialized:** Before being ticked after either being created or reset.
///
/// **Running:** While the function is being executed in the other thread.
///
/// **Succeeded:** When the function returns `true`.
///
/// **Failed:** When the function returns `false`.
///
/// # Children
///
/// None.
///
/// # Examples
///
/// An action node that attempts to subtract two unsigned integers:
///
/// ```
/// # use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};
/// # use aspen::std_nodes::*;
/// # use aspen::Status;
/// const  FIRST:  usize = 10;
/// const  SECOND: usize = 100;
/// static result: AtomicUsize = ATOMIC_USIZE_INIT;
///
/// let mut action = Action::new(||{
///     if FIRST < SECOND {
///         result.store(SECOND - FIRST, Ordering::SeqCst);
///         true
///     } else { false }
/// });
///
/// // Run the node until it completes
/// while !action.tick().is_done() { };
/// assert_eq!(action.status(), Status::Succeeded);
/// assert_eq!(result.load(Ordering::SeqCst), 90);
/// ```
pub struct Action
{
	/// The task which is to be run.
	func: Arc<Fn() -> bool + Send + Sync>,

	/// Channel on which the task will communicate.
	rx: Option<mpsc::Receiver<bool>>,
}
impl Action
{
	/// Creates a new Action node that will execute the given task.
	pub fn new<F>(task: F) -> Node
		where F: Fn() -> bool + Send + Sync + 'static
	{
		let internals = Action {
			func: Arc::new(task),
			rx: None,
		};

		Node::new(internals)
	}

	/// Launches a new worker thread to run the task.
	fn start_thread(&mut self)
	{
		// Create our new channels
		let (tx, rx) = mpsc::channel();

		// Then clone the function so we can move it
		let func_clone = self.func.clone();

		// Finally, boot up the thread
		thread::spawn(move || tx.send((func_clone)()).unwrap() );

		// Store the rx for later use
		self.rx = Some(rx);
	}
}
impl Internals for Action
{
	fn tick(&mut self) -> Status
	{
		if let Some(ref mut rx) = self.rx {
			match rx.try_recv() {
				Ok(true) => Status::Succeeded,
				Ok(false) => Status::Failed,
				Err(TryRecvError::Empty) => Status::Running,
				_ => panic!("Task died before finishing"),
			}
		} else {
			self.start_thread();
			Status::Running
		}
	}

	/// Resets the internal state of this node.
	///
	/// If there is a task currently running, this will block until the task is
	/// completed.
	fn reset(&mut self)
	{
		// I debated what to do here for a while. I could see someone wanting to detach
		// the thread due to time constraints, but it seems to me that it would be better
		// to avoid potential bugs that come from a node only looking like its been
		// fully reset.
		if let Some(ref mut rx) = self.rx {
			rx.recv().unwrap();
		}
		self.rx = None;
	}

	/// Returns the constant string "Action"
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
