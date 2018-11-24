use std::time::{Instant, Duration};
use std::thread;
use std::fmt;

use crate::node::{Node, Tickable};
use crate::status::Status;

/// Main behavior tree struct.
pub struct BehaviorTree<'a, W>
{
	/// Root node of the behavior tree.
	root: Node<'a, W>
}
impl<'a, W> BehaviorTree<'a, W>
{
	/// Create a new behavior tree with the supplied `Node` as the root.
	pub fn new(root: Node<'a, W>) -> BehaviorTree<'a, W>
	{
		BehaviorTree { root: root }
	}

	/// Returns a reference to the root node.
	pub fn root(&self) -> &Node<'a, W>
	{
		&self.root
	}

	/// Tick the behavior tree a single time.
	///
	/// If the tree has already been completed, ticking it again will reset it.
	/// When the tree is reset, it will return an `Initialized` status a single
	/// time.
	pub fn tick(&mut self, world: &mut W) -> Status
	{
		if self.root.status().is_done() {
			debug!("Tree reset via ticking");
			self.root.reset();
			Status::Initialized
		} else { self.root.tick(world) }
	}

	/// Reset the tree to a state identical to before it had ran.
	pub fn reset(&mut self)
	{
		trace!("Tree reset");
		self.root.reset()
	}

	/// Run the behavior tree until it either succeeds or fails.
	///
	/// This makes no guarantees that it will run at the specified frequency. If a single
	/// tick takes longer than the alloted tick time, it will log a warning
	/// unless the specified frequency is infinite.
	///
	/// If the hook is supplied, it will be run after every tick. A reference to this
	/// behavior tree will be supplied as an argument.
	///
	/// NOTE: The only time this will return `Status::Running` is if the frequency is zero
	/// and the behavior tree is running after the first tick.
	pub fn run<F>(&mut self, freq: f64, world: &mut W, mut hook: Option<F>) -> Status
		where F: FnMut(&BehaviorTree<'a, W>)
	{
		// Deal with the "special" case of a zero frequency
		if freq == 0.0f64 {
			debug!("Zero frequency specified, ticking once");
			let status = self.tick(world);
			if let Some(ref mut f) = hook {
				f(self);
			}

			return status;
		}

		// Figure out the time-per-cycle
		let cycle_dur_float = freq.recip();
		let cycle_dur = Duration::new(cycle_dur_float as u64,
		                              (cycle_dur_float.fract() * 1000000000.0f64) as u32);

		// Now, run at the given frequency
		let mut status = Status::Running;
		debug!("Ticking at {}Hz", freq);
		while status == Status::Running {
			let now = Instant::now();

			trace!("Ticking tree");
			status = self.tick(world);
			if let Some(ref mut f) = hook {
				f(self);
			}

			let elapsed = now.elapsed();

			// Sleep for the remaining amount of time
			if !status.is_done() && freq.is_finite() && elapsed < cycle_dur {
				if elapsed < cycle_dur {
					// Really, the Duration would take care of the case where the
					// frequency is infinite. However, specifying a frequency of
					// infinity means running as fast a possible. In that case, I
					// do not want to give this thread an opportunity to sleep at
					// all
					thread::sleep(cycle_dur - elapsed);
				}
				else {
					warn!("Unable to tick at desired frequency: Expected {:?}, elapsed {:?}", cycle_dur, elapsed);
				}
			}
		}

		return status;
	}
}
impl<'a, W> fmt::Display for BehaviorTree<'a, W>
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		write!(f, "BehaviorTree:( status = {:?}, root = {} )", self.root.status(), self.root)
	}
}
