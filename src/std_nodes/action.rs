//! Nodes that cause the execution of tasks.
use std::thread;
use std::sync::Arc;
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;
use node::{Node, Internals};
use status::Status;

pub type Result = ::std::result::Result<(), ()>;

/// A node that manages the execution of tasks in a separate thread.
///
/// This node will launch the supplied function in a separate thread and ticks
/// will monitor the state of that thread. If the supplied function returns
/// `Ok` then the node is considered successful, otherwise it is considered to
/// have failed.
///
/// This node should be the main way of modifying the world state. Note that,
/// despite the function being run in a separate thread, there will usually
/// only be one thread modifying the world.
///
/// # State
///
/// **Initialized:** Before being ticked after either being created or reset.
///
/// **Running:** While the function is being executed in the other thread.
///
/// **Succeeded:** When the function returns `Ok`.
///
/// **Failed:** When the function returns `Err`.
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
///     result.store(SECOND.checked_sub(FIRST).ok_or(())?, Ordering::SeqCst);
///     Ok(())
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
	func: Arc<Fn() -> Result + Send + Sync>,

	/// Channel on which the task will communicate.
	rx: Option<mpsc::Receiver<Result>>,
}
impl Action
{
	/// Creates a new Action node that will execute the given task.
	pub fn new<F>(task: F) -> Node<'static>
		where F: Fn() -> Result + Send + Sync + 'static
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
				Ok(Ok(())) => Status::Succeeded,
				Ok(Err(())) => Status::Failed,
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
	#[allow(unused_must_use)]
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

/// A node that manages the execution of tasks within the ticking thread.
///
/// This node is an alternative to a normal Action node which can be used when
/// the time required to do the task is significantly less than a single tick
/// or if it can be broken down into descrete steps. If the task takes too
/// long, or too many of these nodes are utilized, the ticking rate can be
/// affected.
///
/// # State
///
/// **Initialized:** Before being ticked after either being created or reset.
///
/// **Running:** Generally only if the task can be broken into increments.
///
/// **Succeeded:** When the function returns 'Ok'.
///
/// **Failed:** When the function returns `Err`.
///
/// # Children
///
/// None.
///
/// # Examples
///
/// A short action node that attempts to subtract two unsigned integers:
///
/// ```
/// # use std::cell::Cell;
/// # use aspen::std_nodes::*;
/// # use aspen::Status;
/// let first = 10u32;
/// let second = 100u32;
/// let result = Cell::new(0u32);
///
/// let mut action = InlineAction::new(||{
///     if let Some(n) = second.checked_sub(first) {
///         result.set(n);
///         Status::Succeeded
///     } else { Status::Failed }
/// });
///
/// assert_eq!(action.tick(), Status::Succeeded);
/// assert_eq!(result.get(), 90);
/// ```
pub struct InlineAction<'a>
{
	/// The task which is to be run.
	func: Box<FnMut() -> Status + 'a>,
}
impl<'a> InlineAction<'a>
{
	/// Creates a new `ShortAction` node that will execute the given task.
	pub fn new<F>(task: F) -> Node<'a>
		where F: FnMut() -> Status + 'a
	{
		let internals = InlineAction {
			func: Box::new(task),
		};

		Node::new(internals)
	}
}
impl<'a> Internals for InlineAction<'a>
{
	fn tick(&mut self) -> Status
	{
		(*self.func)()
	}

	fn reset(&mut self)
	{
		// No-op
	}

	/// Returns the constant string "InlineAction"
	fn type_name(&self) -> &'static str
	{
		"InlineAction"
	}
}

#[cfg(test)]
mod test
{
	use std::sync::Mutex;
	use std::sync::mpsc;
	use std::time;
	use std::thread;
	use status::Status;
	use std_nodes::*;

	#[test]
	fn failure()
	{
		let (tx, rx) = mpsc::channel();

		let mrx = Mutex::new(rx);
		let mut action = Action::new(move || {
			// Block until the message is sent, then return its value
			mrx.lock().unwrap().recv().unwrap()
		});

		for _ in 0..5 {
			assert_eq!(action.tick(), Status::Running);
			thread::sleep(time::Duration::from_millis(100));
		}

		tx.send(Err(())).unwrap();

		let mut status = Status::Running;
		while status == Status::Running {
			status = action.tick();
		}

		assert_eq!(status, Status::Failed);
	}

	#[test]
	fn success()
	{
		let (tx, rx) = mpsc::channel();

		let mrx = Mutex::new(rx);
		let mut action = Action::new(move || {
			// Block until the message is sent, then return its value
			mrx.lock().unwrap().recv().unwrap()
		});

		for _ in 0..5 {
			assert_eq!(action.tick(), Status::Running);
			thread::sleep(time::Duration::from_millis(100));
		}

		tx.send(Ok(())).unwrap();

		let mut status = Status::Running;
		while status == Status::Running {
			status = action.tick();
		}

		assert_eq!(status, Status::Succeeded);
	}

	#[test]
	fn inline_failure()
	{
		assert_eq!(InlineAction::new(|| Status::Failed).tick(), Status::Failed);
	}

	#[test]
	fn inline_success()
	{
		assert_eq!(InlineAction::new(|| Status::Succeeded).tick(), Status::Succeeded);
	}

	#[test]
	fn inline_running()
	{
		assert_eq!(InlineAction::new(|| Status::Running).tick(), Status::Running);
	}
}
