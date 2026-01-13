#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env, BytesN};

    #[test]
    fn test_initialize() {
        let env = Env::default();
        let contract_id = env.register_contract(None, EphemeralAccountContract);
        let client = EphemeralAccountContractClient::new(&env, &contract_id);

        let creator = Address::generate(&env);
        let recovery = Address::generate(&env);
        let expiry_ledger = env.ledger().sequence() + 1000;

        // Initialize contract
        client.initialize(&creator, &expiry_ledger, &recovery);

        // Verify status
        let status = client.get_status();
        assert_eq!(status, AccountStatus::Active);

        // Verify not expired
        assert_eq!(client.is_expired(), false);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #1)")] // AlreadyInitialized
    fn test_double_initialize() {
        let env = Env::default();
        let contract_id = env.register_contract(None, EphemeralAccountContract);
        let client = EphemeralAccountContractClient::new(&env, &contract_id);

        let creator = Address::generate(&env);
        let recovery = Address::generate(&env);
        let expiry_ledger = env.ledger().sequence() + 1000;

        // Initialize once
        client.initialize(&creator, &expiry_ledger, &recovery);

        // Try to initialize again - should panic
        client.initialize(&creator, &expiry_ledger, &recovery);
    }

    #[test]
    fn test_record_payment() {
        let env = Env::default();
        let contract_id = env.register_contract(None, EphemeralAccountContract);
        let client = EphemeralAccountContractClient::new(&env, &contract_id);

        let creator = Address::generate(&env);
        let recovery = Address::generate(&env);
        let asset = Address::generate(&env);
        let expiry_ledger = env.ledger().sequence() + 1000;

        // Initialize
        client.initialize(&creator, &expiry_ledger, &recovery);

        // Record payment
        client.record_payment(&100, &asset);

        // Verify status changed
        let status = client.get_status();
        assert_eq!(status, AccountStatus::PaymentReceived);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #3)")] // PaymentAlreadyReceived
    fn test_double_payment() {
        let env = Env::default();
        let contract_id = env.register_contract(None, EphemeralAccountContract);
        let client = EphemeralAccountContractClient::new(&env, &contract_id);

        let creator = Address::generate(&env);
        let recovery = Address::generate(&env);
        let asset = Address::generate(&env);
        let expiry_ledger = env.ledger().sequence() + 1000;

        // Initialize and record first payment
        client.initialize(&creator, &expiry_ledger, &recovery);
        client.record_payment(&100, &asset);

        // Try second payment - should panic
        client.record_payment(&50, &asset);
    }

    #[test]
    fn test_sweep() {
        let env = Env::default();
        let contract_id = env.register_contract(None, EphemeralAccountContract);
        let client = EphemeralAccountContractClient::new(&env, &contract_id);

        let creator = Address::generate(&env);
        let recovery = Address::generate(&env);
        let asset = Address::generate(&env);
        let destination = Address::generate(&env);
        let expiry_ledger = env.ledger().sequence() + 1000;

        // Setup
        client.initialize(&creator, &expiry_ledger, &recovery);
        client.record_payment(&100, &asset);

        // Create dummy auth signature
        let auth_sig = BytesN::from_array(&env, &[0u8; 64]);

        // Execute sweep
        client.sweep(&destination, &auth_sig);

        // Verify status
        let status = client.get_status();
        assert_eq!(status, AccountStatus::Swept);
    }

    #[test]
    fn test_expiration() {
        let env = Env::default();
        env.mock_all_auths();
        
        let contract_id = env.register_contract(None, EphemeralAccountContract);
        let client = EphemeralAccountContractClient::new(&env, &contract_id);

        let creator = Address::generate(&env);
        let recovery = Address::generate(&env);
        let expiry_ledger = env.ledger().sequence() + 10;

        // Initialize
        client.initialize(&creator, &expiry_ledger, &recovery);

        // Check not expired yet
        assert_eq!(client.is_expired(), false);

        // Advance ledger past expiry
        env.ledger().set_sequence_number(expiry_ledger + 1);

        // Check is expired
        assert_eq!(client.is_expired(), true);

        // Execute expiration
        client.expire();

        // Verify status
        let status = client.get_status();
        assert_eq!(status, AccountStatus::Expired);
    }

    #[test]
    fn test_get_info() {
        let env = Env::default();
        let contract_id = env.register_contract(None, EphemeralAccountContract);
        let client = EphemeralAccountContractClient::new(&env, &contract_id);

        let creator = Address::generate(&env);
        let recovery = Address::generate(&env);
        let asset = Address::generate(&env);
        let expiry_ledger = env.ledger().sequence() + 1000;

        // Initialize and record payment
        client.initialize(&creator, &expiry_ledger, &recovery);
        client.record_payment(&100, &asset);

        // Get info
        let info = client.get_info();

        // Verify
        assert_eq!(info.creator, creator);
        assert_eq!(info.status, AccountStatus::PaymentReceived);
        assert_eq!(info.expiry_ledger, expiry_ledger);
        assert_eq!(info.payment_received, true);
        assert_eq!(info.payment_amount, Some(100));
    }
}