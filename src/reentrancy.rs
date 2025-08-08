use stylus_sdk::{
    alloy_primitives::U256,
    prelude::*,
    storage::{StorageBool},
};

// Custom error for reentrancy attempts
#[derive(SolidityError)]
pub enum ReentrancyGuardError {
    ReentrantCall,
}

#[solidity_storage]
pub struct ReentrancyGuard {
    // Tracks if a function is currently executing
    entered: StorageBool,
}

impl ReentrancyGuard {
    /// Initializes the guard, setting entered to false
    pub fn init(&mut self) {
        self.entered.set(false);
    }

    /// Ensures the function is not called reentrantly
    pub fn non_reentrant<T, F: FnOnce() -> Result<T, ReentrancyGuardError>>(
        &mut self,
        f: F,
    ) -> Result<T, ReentrancyGuardError> {
        if self.entered.get() {
            return Err(ReentrancyGuardError::ReentrantCall);
        }
        self.entered.set(true);
        let result = f();
        self.entered.set(false);
        result
    }
}