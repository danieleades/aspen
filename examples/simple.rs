#[macro_use]
extern crate aspen;

use std::sync::{Arc, Mutex};
use std::{thread, time};
use aspen::{BehaviorTree, Status};

const INPUT_A: u32 = 5;
const INPUT_B: u32 = 7;

#[derive(Debug, Default)]
struct WorldState
{
	add_res: Option<u32>,
	sub_res: Option<u32>,
}

// Entry point of the program
fn main()
{
	// The sync elements are required because the Action node works in a
	// separate thread. Otherwise should not be necessary.
	let mut world_state: Arc<Mutex<WorldState>> = Default::default();

	// Create the tree - sleep to simulate work
	let root = Sequence!{
		// Addition node
		Action!{ do_add },

		// Condition node to check if we can safely do the subtraction
		Condition!{ |_| INPUT_B > INPUT_A },

		// Subtraction node. Only runs if the condition is successful. This one
		// doesn't do a long task (there is not sleep statement), so we can use
		// a `InlineAction` node, which will not boot up a new thread.
		InlineAction!{ do_sub }
	};

	// Put it all in a tree, print it, and run it
	let mut tree = BehaviorTree::new(root);
	println!("{}", tree);
	let res = tree.run(4.0, &mut world_state, Some(hook));

	println!("\nTree finished: {:?}", res);
	println!("\nINPUT_A: {}\nINPUT_B: {}", INPUT_A, INPUT_B);
	println!("{:?}", world_state);
}

fn do_add(state: Arc<Mutex<WorldState>>) -> Status
{
	let mut locked_state = state.lock().unwrap();

	// Sleep to simulate doing a lot of work
	thread::sleep(time::Duration::from_secs(1));
	locked_state.add_res = INPUT_A.checked_add(INPUT_B);

	if locked_state.add_res.is_some() {
		Status::Succeeded
	} else { Status::Failed }
}

fn do_sub(state: &mut Arc<Mutex<WorldState>>) -> Status
{
	// We know that the subtraction will be valid because of the condition node
	let mut locked_state = state.lock().unwrap();
	locked_state.sub_res = Some(INPUT_B - INPUT_A);
	Status::Succeeded
}

// Display the tree after each tick
fn hook<W>(tree: &BehaviorTree<W>)
{
	println!("{}", tree);
}
