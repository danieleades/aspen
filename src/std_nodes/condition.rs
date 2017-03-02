//! Nodes whose status is determined by a function
use std::sync::Arc;
use node::Node;
use status::Status;

/// A node whose success depends on a function that can be run in a single tick
pub struct Condition<T: Send + Sync + 'static>
{
	/// Function that is performed to determine the node's status
	///
	/// A return value of `true` means success and a return value of `false` means failure
	func: Box<Fn(&Arc<T>) -> bool>,

	/// Return status of the last tick
	status: Status,
}
impl<T: Send + Sync + 'static> Condition<T>
{
	/// Constructs a new Condition node
	///
	/// If the functio returns `true`, then then node succeeds. Otherwise the node fails.
	pub fn new(func: Box<Fn(&Arc<T>) -> bool>) -> Condition<T>
	{
		Condition {
			func: func,
			status: Status::Running,
		}
	}
}
impl<T: Send + Sync + 'static> Node<T> for Condition<T>
{
	fn tick(&mut self, world: &Arc<T>) -> Status
	{
		// If we've already run, don't run again
		if self.status != Status::Running {
			return self.status;
		}

		// Otherwise, run the function
		self.status = if (*self.func)(world) {
			Status::Succeeded
		} else {
			Status::Failed
		};

		self.status
	}

	fn reset(&mut self)
	{
		self.status = Status::Running;
	}

	fn status(&self) -> Status
	{
		self.status
	}
}

#[cfg(test)]
mod test
{
	use std::sync::Arc;
	use std::sync::atomic::{AtomicBool, Ordering};
	use node::Node;
	use status::Status;
	use std_nodes::*;

	fn condition(world: &Arc<AtomicBool>) -> bool
	{
		world.load(Ordering::SeqCst)
	}

	#[test]
	fn failure()
	{
		let world = Arc::new(AtomicBool::new(false));
		let mut cond = Condition::new(Box::new(condition));
		assert_eq!(cond.tick(&world), Status::Failed);
	}

	#[test]
	fn success()
	{
		let world = Arc::new(AtomicBool::new(true));
		let mut cond = Condition::new(Box::new(condition));
		assert_eq!(cond.tick(&world), Status::Succeeded);
	}
}
