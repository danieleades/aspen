//! Nodes that have a single child and whose status is some function of the
//! child's status.
use std::sync::Arc;
use node::{Node, Internals, IdType};
use status::Status;

/// Implements a generic Decorator node
pub struct Decorator<T: Send + Sync + 'static>
{
	/// Function that is performed on the child's status
	func: Box<Fn(Status) -> Status>,

	/// Child node
	child: Node<T>,
}
impl<T: Send + Sync + 'static> Decorator<T>
{
	/// Creates a new Decorator node with the given child and function
	pub fn new(child: Node<T>, func: Box<Fn(Status) -> Status>) -> Node<T>
	{
		let internals = Decorator { func: func, child: child };
		Node::new(internals)
	}
}
impl<T: Send + Sync + 'static> Internals<T> for Decorator<T>
{
	fn tick(&mut self, world: &Arc<T>) -> Status
	{
		// If the child has already run, this shouldn't change results since it will
		// just return its last status
		let child_status = self.child.tick(world);

		// Now run it through the function
		(*self.func)(child_status);
	}

	fn reset(&mut self)
	{
		self.child.reset();
	}

	fn children(&self) -> Vec<&Node<T>>
	{
		vec![&self.child]
	}

	fn children_ids(&self) -> Vec<IdType>
	{
		vec![self.child.id()]
	}

	fn type_name() -> &str
	{
		"Decorator"
	}
}

/// Implements a node that will reset its child after the child succeeds or fails
pub struct Reset<T: Send + Sync + 'static>
{
	/// Child node
	child: Node<T>,

	/// Optional number of times to do the reset
	attempt_limit: Option<u32>,

	/// Number of times the child has been reset
	attempts: u32,
}
impl<T: Send + Sync + 'static> Reset<T>
{
	/// Creates a new Reset node that will reset the child indefinitely
	pub fn new(child: Node<T>) -> Node<T>
	{
		let internals = Reset {
			child: child,
			attempt_limit: None,
			attempts: 0,
		};
		Node::new(internals)
	}

	/// Creates a new Reset node that will reset the child a limited number of times
	pub fn with_limit(child: Node<T>, limit: u32) -> Node<T>
	{
		let internals = Reset {
			child: child,
			attempt_limit: Some(limit),
			attempts: 0,
		};
		Node::new(internals)
	}
}
impl<T: Send + Sync + 'static> Internals<T> for Reset<T>
{
	fn tick(&mut self, world: &Arc<T>) -> Status
	{
		// First, get the last status of the child
		let child_last_status = self.child.status();

		// If the child wasn't Running (or already reset), we need to reset it... if the count allows
		let reset = child_last_status.is_done()
		            && (self.attempt_limit == None
		            || self.attempt_limit.unwrap() > self.attempts);
		if reset {
			// Theoretically, this could overflow if there is no attempt limit. But, if
			// does, then the user really didn't plan - the node should never tick that
			// many times in any situation.
			self.child.reset();
			self.attempts += 1;
		}

		// Now tick the child
		self.child.tick(world);
	}

	fn reset(&mut self)
	{
		// Reset our attempt count
		self.attempts = 0;

		// Reset the child
		self.child.reset();
	}

	fn children(&self) -> Vec<&Node<T>>
	{
		vec![&self.child]
	}

	fn children_ids(&self) -> Vec<IdType>
	{
		vec![self.child.id()]
	}

	fn type_name() -> &str
	{
		"Reset"
	}
}

/// Implements a node that will reset its child after the child fails
pub struct Retry<T: Send + Sync + 'static>
{
	/// Child node
	child: Node<T>,

	/// Optional number of times to do the reset
	attempt_limit: Option<u32>,

	/// Number of times the child has been reset
	attempts: u32,
}
impl<T: Send + Sync + 'static> Retry<T>
{
	/// Creates a new Retry node that will retry the child indefinitely
	pub fn new(child: Node<T>) -> Node<T>
	{
		let internals = Retry {
			child: child,
			attempt_limit: None,
			attempts: 0,
		};
		Node::new(internals)
	}

	/// Creates a new Retry node that will retry the child a limited number of times
	pub fn with_limit(child: Node<T>, limit: u32) -> Node<T>
	{
		let internals = Retry {
			child: child,
			attempt_limit: Some(limit),
			attempts: 0,
		};
		Node::new(internals)
	}
}
impl<T: Send + Sync + 'static> Internals<T> for Retry<T>
{
	fn tick(&mut self, world: &Arc<T>) -> Status
	{
		// First, get the last status of the child
		let child_last_status = self.child.status();

		// If the child failed, we need to retry it... if the count allows
		let reset = child_last_status == Status::Failed
		            && (self.attempt_limit == None
		            || self.attempt_limit.unwrap() > self.attempts);
		if reset {
			// Theoretically, this could overflow if there is no attempt limit. But, if
			// does, then the user really didn't plan - the node should never tick that
			// many times in any situation.
			self.child.reset();
			self.attempts += 1;
		}

		// Now tick the child
		self.child.tick(world);
	}

	fn reset(&mut self)
	{
		// Reset our own status
		self.attempts = 0;

		// Reset the child
		self.child.reset();
	}

	fn children(&self) -> Vec<&Node<T>>
	{
		vec![&self.child]
	}

	fn children_ids(&self) -> Vec<IdType>
	{
		vec![self.child.id()]
	}

	fn type_name() -> &str
	{
		"Retry"
	}
}

#[cfg(test)]
mod test
{
	use std::sync::Arc;
	use std::sync::atomic::AtomicBool;
	use node::Node;
	use status::Status;
	use std_nodes::*;

	fn rotate(s: Status) -> Status
	{
		match s {
			Status::Initialized => Status::Running,
			Status::Running => Status::Succeeded,
			Status::Succeeded => Status::Failed,
			Status::Failed => Status::Initialized,
		}
	}

	#[test]
	fn check_decorator()
	{
		// Use an atomic as the world (doesn't actually get used)
		let world = Arc::new(AtomicBool::new(true));

		// Test the first rotation
		let suc_child = YesTick::new(Status::Succeeded);
		let mut suc_dec = Decorator::new(suc_child, Box::new(rotate));
		let suc_status = suc_dec.tick(&world);
		drop(suc_dec);
		assert_eq!(suc_status, rotate(Status::Succeeded));

		// Test the second rotation
		let run_child = YesTick::new(Status::Running);
		let mut run_dec = Decorator::new(run_child, Box::new(rotate));
		let run_status = run_dec.tick(&world);
		drop(run_dec);
		assert_eq!(run_status, rotate(Status::Running));

		// Test the final rotation
		let fail_child = YesTick::new(Status::Failed);
		let mut fail_dec = Decorator::new(fail_child, Box::new(rotate));
		let fail_status = fail_dec.tick(&world);
		drop(fail_dec);
		assert_eq!(fail_status, rotate(Status::Failed));
	}

	#[test]
	fn check_reset()
	{
		// Use an atomic as the world (not actually used)
		let world = Arc::new(AtomicBool::new(true));

		// No good way to test ticking indefinitely, so we'll tick a
		// specified number of times
		let child = CountedTick::new(Status::Succeeded, 5, true);
		let mut reset = Reset::with_limit(child, 5);

		// Tick it five times
		let mut status = Status::Running;
		for _ in 0..5 {
			status = reset.tick(&world);
		}

		// Drop the node so the testing nodes can panic
		drop(reset);

		// Now make sure we got the right output
		assert_eq!(status, Status::Succeeded);
	}

	#[test]
	fn check_retry()
	{
		// Use an atomic for the world (because necessary)
		let world = Arc::new(AtomicBool::new(true));

		// We can test to make sure that the "indefinite" only ticks while failed
		let child1 = CountedTick::new(Status::Succeeded, 1, true);
		let mut retry1 = Retry::new(child1);
		let mut status1 = Status::Running;
		while status1 == Status::Running { status1 = retry1.tick(&world); };
		drop(retry1);
		assert_eq!(status1, Status::Succeeded);

		// No good way to test infinite retrying, so use a limited number
		let child2 = CountedTick::new(Status::Failed, 5, true);
		let mut retry2 = Retry::with_limit(child2, 5);
		let mut status2 = Status::Running;
		for _ in 0..5 { status2 = retry2.tick(&world); }
		drop(retry2);
		assert_eq!(status2, Status::Failed);
	}
}
