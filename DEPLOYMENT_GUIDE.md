# Integration Checklist & Deployment Guide

## Pre-Deployment Checklist

### Contract Implementation
- [x] Ed25519 signature verification implemented
- [x] Nonce mechanism for replay attack prevention
- [x] Error types added for signature failures
- [x] Storage module for authorized signer and nonce
- [x] Initialize function implemented
- [x] Execute sweep updated with nonce increment
- [x] Tests added for all flows

### Documentation
- [x] Signature format documented
- [x] Message construction specified
- [x] Examples provided (TypeScript, Python, Rust)
- [x] Implementation summary created
- [x] Quick reference guide created
- [x] Flow diagrams created

## Files Changed

### Modified Files
```
contracts/sweep_controller/src/
  ├─ errors.rs          (Updated with new error types)
  ├─ lib.rs             (Added initialize, updated execute_sweep)
  └─ authorization.rs   (Implemented real signature verification)
```

### New Files
```
contracts/sweep_controller/src/
  └─ storage.rs         (Created - storage for signer & nonce)

docs/
  ├─ SIGNATURE_FORMAT.md         (New - signature spec & examples)
  ├─ QUICK_REFERENCE.md          (New - quick start guide)
  ├─ FLOW_DIAGRAM.md             (New - architecture diagrams)
  └─ IMPLEMENTATION_SUMMARY.md   (New - full summary)
```

## Off-Chain System Integration Steps

### 1. Key Management Setup
```
Step 1a: Generate Ed25519 keypair
  openssl genpkey -algorithm ed25519 -out signer_private.pem
  
Step 1b: Extract public key (32 bytes)
  openssl pkey -outform DER -pubout -in signer_private.pem \
    | tail -c 32 > signer_public.key
  
Step 1c: Securely store private key
  - Use HSM (Hardware Security Module)
  - Or encrypted key management service
  - Never commit to version control
```

### 2. Contract Initialization
```
Step 2a: Read public key bytes
  public_key_bytes = read_binary("signer_public.key")  // 32 bytes
  
Step 2b: Convert to BytesN<32>
  authorized_signer: BytesN<32> = public_key_bytes
  
Step 2c: Call initialize()
  sweep_controller.initialize(&authorized_signer)
  
Step 2d: Verify initialization succeeded
  - Check contract storage contains public key
  - Verify nonce is initialized to 0
  - Test signature verification works
```

### 3. Signature Generation Workflow
```
For each sweep request:

Step 3a: Get current state
  current_nonce = get_nonce_from_contract()
  contract_id = get_contract_id()
  timestamp = current_unix_timestamp()
  destination = user_requested_destination
  
Step 3b: Construct message
  message = concatenate([
    destination.to_xdr(),
    current_nonce.to_be_bytes(),
    contract_id.to_xdr(),
    timestamp.to_be_bytes()
  ])
  
Step 3c: Hash message
  message_hash = SHA256(message)  // 32 bytes
  
Step 3d: Sign hash
  signature = ed25519_sign(message_hash, private_key)  // 64 bytes
  
Step 3e: Verify signature locally (optional)
  assert(ed25519_verify(public_key, message_hash, signature))
```

### 4. Transaction Submission
```
Step 4a: Prepare sweep transaction
  tx = SweepController.execute_sweep(
    ephemeral_account=account_address,
    destination=destination_address,
    auth_signature=signature
  )
  
Step 4b: Submit transaction
  result = submit_transaction(tx)
  
Step 4c: Check result
  if result.success:
    - Sweep completed
    - Nonce has been incremented on-chain
    - Emit success event
  else:
    - Check error type
    - If SignatureVerificationFailed: verify message construction
    - If InvalidNonce: refresh state and retry
    - Otherwise: handle error appropriately
```

### 5. Error Handling
```
Error Scenarios:

1. SignatureVerificationFailed
   └─ Action: 
     ├─ Verify message construction matches spec
     ├─ Check nonce is current
     ├─ Verify private key matches public key on-chain
     └─ Re-generate signature and retry

2. AuthorizedSignerNotSet
   └─ Action:
     ├─ Initialize contract with initialize()
     ├─ Verify public key was stored
     └─ Retry sweep

3. InvalidNonce
   └─ Action:
     ├─ Query contract for current nonce
     ├─ Regenerate message with correct nonce
     ├─ Create new signature
     └─ Resubmit

4. AccountNotReady
   └─ Action:
     ├─ Verify payment has been received
     ├─ Wait for payment confirmation
     └─ Retry
```

## Implementation Testing

### Unit Test Execution
```bash
# Test all components
cd contracts/sweep_controller
cargo test

# Run specific test
cargo test test_initialize_sweep_controller

# Run with logs
RUST_LOG=debug cargo test
```

### Integration Testing

#### Test 1: Full Sweep Flow
```
1. Initialize contract with test keypair
2. Create ephemeral account with payment
3. Generate valid signature
4. Execute sweep with valid signature
5. Verify sweep completed and nonce incremented
6. Try to reuse same signature (should fail)
```

#### Test 2: Signature Generation Verification
```
1. Use TypeScript example to generate signature
2. Use Python example to verify signature
3. Use Rust example to validate both
4. All three should produce same results
```

#### Test 3: Wrong Signer Rejection
```
1. Initialize with signer A public key
2. Generate signature with signer B private key
3. Submit sweep with B's signature
4. Verify SignatureVerificationFailed error
```

#### Test 4: Replay Attack Prevention
```
1. Submit valid sweep transaction → succeeds, nonce → 1
2. Extract same signature from first transaction
3. Submit same signature again → fails with verification error
4. Regenerate message with new nonce → succeeds
```

## Production Deployment Checklist

### Security Review
- [ ] Code audit completed
- [ ] Cryptographic library versions verified
- [ ] No hardcoded keys in code
- [ ] Error messages don't leak sensitive info
- [ ] Nonce overflow handled (u64 is sufficient)

### Infrastructure
- [ ] Key management system operational
- [ ] HSM available for private key storage (recommended)
- [ ] Secure key rotation process defined
- [ ] Key backup and recovery procedures in place
- [ ] Access controls on key material

### Monitoring & Alerting
- [ ] SignatureVerificationFailed events logged
- [ ] Nonce increments tracked
- [ ] Failed sweep attempts monitored
- [ ] Alerts set for unusual patterns
- [ ] Audit trail maintained

### Documentation
- [ ] Team trained on signature generation
- [ ] Runbooks created for troubleshooting
- [ ] Emergency procedures documented
- [ ] Incident response plan in place

### Rollout Strategy
- [ ] Deploy to testnet first
- [ ] Verify with test signatures
- [ ] Run integration tests
- [ ] Monitor for 24 hours
- [ ] Promote to mainnet
- [ ] Have rollback plan ready

## Operational Procedures

### Daily Operations
```
Each sweep:
1. Query current nonce from contract
2. Generate signature with current nonce
3. Submit sweep transaction
4. Verify success
5. Monitor for failures
```

### Key Rotation
```
When rotating keys:
1. Generate new Ed25519 keypair
2. Deploy new public key to contract via initialize()
3. Update off-chain system to use new private key
4. Test with new key
5. Decommission old key (after safety period)
```

### Disaster Recovery
```
If private key compromised:
1. STOP all sweep operations immediately
2. Generate new keypair
3. Deploy new public key via initialize()
4. Restart sweep operations with new key
5. Audit all transactions from compromise time
```

### Monitoring Dashboard
```
Key metrics to track:
- Signature verification success rate
- Signature verification failure rate
- Nonce increment frequency
- Average time to generate signature
- Failed sweep attempts per hour
- Error distribution
```

## Compliance & Audit

### Logging
```
Log all of the following:
- Signature verification attempts (success/failure)
- Sweep completions with signatures
- Nonce increments
- Contract initialization
- Error conditions
```

### Audit Trail
```
Maintain record of:
- All sweep transactions
- Corresponding signatures used
- Nonce values at time of sweep
- Verification results
- Timestamps of all operations
```

### Compliance Reports
```
Generate reports for:
- Failed authentication attempts
- Unusual sweep patterns
- Key rotation events
- Access to key material
- System availability metrics
```

## Rollback Procedure

If issues discovered post-deployment:

```
1. Stop sweep operations
2. Identify issue (e.g., signature verification failing)
3. Verify contract on testnet with fix
4. Deploy fix to mainnet
5. Run integration tests
6. Resume operations with new contract version
```

## Performance Considerations

### Gas/Fee Estimates
```
Operation                          | Approximate Cost
─────────────────────────────────────────────────────
initialize()                       | ~5,000 ops
execute_sweep()                    | ~20,000 ops (including verification)
Signature verification             | ~10,000 ops (crypto intensive)
Nonce increment                    | ~1,000 ops
```

### Optimization Tips
```
1. Batch sweeps when possible
2. Cache current nonce locally
3. Pre-validate signatures before submission
4. Monitor gas usage per operation
5. Consider multi-sig patterns for future scaling
```

## Support & Troubleshooting

### Common Issues

**Issue: "SignatureVerificationFailed"**
- Check: Nonce is current
- Check: Message construction matches spec
- Check: Private/public keypair is matching
- Solution: See error handling section

**Issue: "AuthorizedSignerNotSet"**
- Check: Contract initialization completed
- Solution: Run initialize() with correct public key

**Issue: Sweep never succeeds**
- Check: Ephemeral account has payment
- Check: Signature is valid
- Solution: Run full integration test

### Getting Help

1. Check SIGNATURE_FORMAT.md for message construction
2. Review implementation examples in your language
3. Use provided test vectors to debug
4. Enable debug logging in contract
5. Compare with working reference implementation

## Next Steps

1. **Immediate**: Deploy to testnet and test
2. **Week 1**: Complete security review
3. **Week 2**: Train operations team
4. **Week 3**: Deploy to mainnet with monitoring
5. **Ongoing**: Monitor and optimize
