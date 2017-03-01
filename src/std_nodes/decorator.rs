//! Nodes that have a single child and whose status is some function of the
//! child's status.
use std::sync::Arc;
use node::Node;
use status::Status;

/// Implements a generic Decorator node
pub struct Decorator<T: Send + Sync + 'static>
{
	/// Function that is performed on the child's status
	func: Box<Fn(Status) -> Status>,

	/// Child node
	child: Box<Node<T>>,
}
impl<T: Send + Sync + 'static> Decorator<T>
{
	/// Creates a new Decorator node with the given child and function
	pub fn new(child: Box<Node<T>>, func: Box<Fn(Status) -> Status>) -> Decorator<T>
	{
		Decorator {
			func: func,
			child: child,
		}
	}
}
impl<T: Send + Sync + 'static> Node<T> for Decorator<T>
{
	fn tick(&mut self, world: &Arc<T>) -> Status
	{
		// If the child has already run, this shouldn't change results since it will
		// just return its last status
		let child_status = (*self.child).tick(world);

		// Now run it through the function
		(*self.func)(child_status)
	}

	fn reset(&mut self)
	{
		// Reset our child
		(*self.child).reset();
	}

	fn status(&self) -> Status
	{
		(*self.func)(self.child.status())
	}
}

/// Implements a node that will reset its child after the child succeeds or fails
pub struct Reset<T: Send + Sync + 'static>
{
	/// Child node
	child: Box<Node<T>>,

	/// Optional number of times to do the reset
	attempt_limit: Option<u32>,

	/// Number of times the child has been reset
	attempts: u32,
}
impl<T: Send + Sync + 'static> Reset<T>
{
	/// Creates a new Reset node that will reset the child indefinitely
	pub fn new(child: Box<Node<T>>) -> Reset<T>
	{
		Reset {
			child: child,
			attempt_limit: None,
			attempts: 0,
		}
	}

	/// Creates a new Reset node that will reset the child a limited number of times
	pub fn with_limit(child: Box<Node<T>>, limit: u32) -> Reset<T>
	{
		Reset {
			child: child,
			attempt_limit: Some(limit),
			attempts: 0,
		}
	}
}
impl<T: Send + Sync + 'static> Node<T> for Reset<T>
{
	fn tick(&mut self, world: &Arc<T>) -> Status
	{
		// First, get the last status of the child
		let child_last_status = (*self.child).status();

		// If the child wasn't Running, we need to reset it... if the count allows
		let reset = child_last_status != Status::Running
		            && (self.attempt_limit == None
		            || self.attempt_limit.unwrap() > self.attempts);
		if reset {
			// Theoretically, this could overflow if there is no attempt limit. But, if
			// does, then the user really didn't plan - the node should never tick that
			// many times in any situation.
			(*self.child).reset();
			self.attempts += 1;
		}

		// Now tick the child
		(*self.child).tick(world)
	}

	fn reset(&mut self)
	{
		// Reset our own status
		self.attempts = 0;

		// Reset the child
		(*self.child).reset();
	}

	fn status(&self) -> Status
	{
		// Not sure if this should report Running if we haven't hit our reset limit
		(*self.child).status()
	}
}

/// Implements a node that will reset its child after the child fails
pub struct Retry<T: Send + Sync + 'static>
{
	/// Child node
	child: Box<Node<T>>,

	/// Optional number of times to do the reset
	attempt_limit: Option<u32>,

	/// Number of times the child has been reset
	attempts: u32,
}
impl<T: Send + Sync + 'static> Retry<T>
{
	/// Creates a new Retry node that will retry the child indefinitely
	pub fn new(child: Box<Node<T>>) -> Retry<T>
	{
		Retry {
			child: child,
			attempt_limit: None,
			attempts: 0,
		}
	}

	/// Creates a new Retry node that will retry the child a limited number of times
	pub fn with_limit(child: Box<Node<T>>, limit: u32) -> Retry<T>
	{
		Retry {
			child: child,
			attempt_limit: Some(limit),
			attempts: 0,
		}
	}
}
impl<T: Send + Sync + 'static> Node<T> for Retry<T>
{
	fn tick(&mut self, world: &Arc<T>) -> Status
	{
		// First, get the last status of the child
		let child_last_status = (*self.child).status();

		// If the child failed, we need to retry it... if the count allows
		let reset = child_last_status == Status::Failed
		            && (self.attempt_limit == None
		            || self.attempt_limit.unwrap() > self.attempts);
		if reset {
			// Theoretically, this could overflow if there is no attempt limit. But, if
			// does, then the user really didn't plan - the node should never tick that
			// many times in any situation.
			(*self.child).reset();
			self.attempts += 1;
		}

		// Now tick the child
		(*self.child).tick(world)
	}

	fn reset(&mut self)
	{
		// Reset our own status
		self.attempts = 0;

		// Reset the child
		(*self.child).reset();
	}

	fn status(&self) -> Status
	{
		// Should this report Running if the child is Failed?
		(*self.child).status()
	}
}
