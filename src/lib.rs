#[cfg(feature = "lcm")]
extern crate lcm;
#[cfg(feature = "lcm")]
pub use node_message::NodeMsg;
#[cfg(feature = "lcm")]
mod node_message;

#[cfg(feature = "lcm")]
/// Slightly gross hack to simulate C++'s "friend methods"
trait Rootable
{
	fn set_root(&mut self, root: bool);
}

pub use status::Status;
pub use bt::BehaviorTree;

mod status;
pub mod node;
mod bt;

pub mod std_nodes;
