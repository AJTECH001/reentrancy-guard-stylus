use stylus_sdk::{
    alloy_primitives::{Address, U256},
    prelude::*,
    storage::{StorageMap, StorageU256},
    msg,
};
use reentrancy::ReentrancyGuard;

#[derive(SolidityError)]
pub enum VaultError {
    InsufficientBalance,
    ReentrancyGuardError(ReentrancyGuardError),
}

#[solidity_storage]
pub struct Vault {
    // User balances
    balances: StorageMap<Address, StorageU256>,
    // Reentrancy guard
    guard: ReentrancyGuard,
}

#[external]
impl Vault {
    /// Initializes the contract
    pub fn init(&mut self) {
        self.guard.init();
    }

    /// Deposits funds into the vault
    pub fn deposit(&mut self) -> Result<(), VaultError> {
        let caller = msg::sender();
        let amount = msg::value();
        let current_balance = self.balances.get(caller);
        self.balances.insert(caller, current_balance + amount);
        Ok(())
    }

    /// Vulnerable withdraw function (no reentrancy protection)
    pub fn withdraw_unsafe(&mut self, amount: U256) -> Result<(), VaultError> {
        let caller = msg::sender();
        let current_balance = self.balances.get(caller);
        if current_balance < amount {
            return Err(VaultError::InsufficientBalance);
        }
        // External call before state update (vulnerable to reentrancy)
        if !msg::call(caller, amount, vec![])? {
            panic!("Transfer failed");
        }
        self.balances.insert(caller, current_balance - amount);
        Ok(())
    }

    /// Safe withdraw function with reentrancy protection
    pub fn withdraw(&mut self, amount: U256) -> Result<(), VaultError> {
        self.guard
            .non_reentrant(|| {
                let caller = msg::sender();
                let current_balance = self.balances.get(caller);
                if current_balance < amount {
                    return Err(VaultError::InsufficientBalance);
                }
                // External call before state update, but guarded
                if !msg::call(caller, amount, vec![])? {
                    panic!("Transfer failed");
                }
                self.balances.insert(caller, current_balance - amount);
                Ok(())
            })
            .map_err(|e| VaultError::ReentrancyGuardError(e))
    }

    /// Get balance of an address
    pub fn get_balance(&self, address: Address) -> U256 {
        self.balances.get(address)
    }
}