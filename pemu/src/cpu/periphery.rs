use std::sync::Arc;
use parking_lot::Mutex;

pub type KeyboardOutput = Arc<Mutex<u8>>;

pub fn keyboard_output() -> KeyboardOutput {
    Arc::new(Mutex::new(0))
}