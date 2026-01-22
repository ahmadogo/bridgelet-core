# Sweep Authorization Flow Diagram

## Complete Authorization Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                         OFF-CHAIN SYSTEM                             │
│                                                                       │
│  1. User initiates sweep request                                     │
│     ├─ Destination wallet address                                    │
│     └─ Amount to sweep                                               │
│                                                                       │
│  2. Query contract state                                             │
│     ├─ Current nonce                                                 │
│     ├─ Contract ID                                                   │
│     └─ Current timestamp                                             │
│                                                                       │
│  3. Construct message                                                │
│     ┌────────────────────────────────────────────────────┐          │
│     │ SHA256(                                            │          │
│     │   destination_xdr ||                               │          │
│     │   nonce_bytes ||                                   │          │
│     │   contract_id_xdr ||                               │          │
│     │   timestamp_bytes                                  │          │
│     │ )                                                  │          │
│     └────────────────────────────────────────────────────┘          │
│     Result: 32-byte message hash                                     │
│                                                                       │
│  4. Sign with Ed25519 private key                                    │
│     ├─ Input: 32-byte message hash                                   │
│     ├─ Key: private_key (32 bytes)                                   │
│     └─ Output: signature (64 bytes)                                  │
│                                                                       │
│  5. Prepare transaction                                              │
│     execute_sweep(                                                   │
│       ephemeral_account,                                             │
│       destination,                                                   │
│       signature                                                      │
│     )                                                                │
│                                                                       │
└──────────────────────────┬──────────────────────────────────────────┘
                           │ Submit transaction
                           ↓
┌──────────────────────────────────────────────────────────────────────┐
│                     ON-CHAIN VERIFICATION                             │
│                                                                        │
│  1. Receive call to execute_sweep()                                   │
│     ├─ ephemeral_account: Address                                     │
│     ├─ destination: Address                                           │
│     └─ signature: BytesN<64>                                          │
│                                                                        │
│  2. Call verify_sweep_auth()                                          │
│                                                                        │
│  3. Retrieve authorized signer from storage                           │
│     └─ public_key: BytesN<32>                                         │
│        Status: ✗ AuthorizedSignerNotSet if not initialized           │
│                                                                        │
│  4. Get current state for message reconstruction                      │
│     ├─ current_nonce: u64                                             │
│     ├─ contract_id: Address                                           │
│     └─ timestamp: u64                                                 │
│                                                                        │
│  5. Reconstruct message (identical to off-chain)                      │
│     ┌────────────────────────────────────────────────────┐           │
│     │ SHA256(                                            │           │
│     │   destination_xdr ||                               │           │
│     │   nonce_bytes ||                                   │           │
│     │   contract_id_xdr ||                               │           │
│     │   timestamp_bytes                                  │           │
│     │ )                                                  │           │
│     └────────────────────────────────────────────────────┘           │
│     Result: message_hash                                              │
│                                                                        │
│  6. Verify signature                                                  │
│     env.crypto().ed25519_verify(                                      │
│       public_key,                                                     │
│       message_hash,                                                   │
│       signature                                                       │
│     )                                                                 │
│                                                                        │
│     Possible outcomes:                                                │
│     ✓ Verification succeeds                                           │
│       └─ Continue to step 7                                           │
│     ✗ Verification fails                                              │
│       └─ Return SignatureVerificationFailed error                     │
│                                                                        │
│  7. Increment nonce (replay attack prevention)                        │
│     nonce = nonce + 1                                                 │
│     ├─ Previous signature becomes invalid                             │
│     ├─ Message hash would be different next time                      │
│     └─ Prevents reusing same signature                                │
│                                                                        │
│  8. Call ephemeral account contract                                   │
│     account.sweep(destination, signature)                             │
│     └─ Validates account state and authorization                      │
│                                                                        │
│  9. Execute token transfer                                            │
│     token.transfer(ephemeral_account, destination, amount)            │
│                                                                        │
│  10. Emit SweepCompleted event                                        │
│      └─ Audit trail of completed sweep                                │
│                                                                        │
└────────────────────────────────────────────────────────────────────────┘
```

## Replay Attack Prevention Mechanism

```
First Sweep (Nonce = 0):
  Message_1 = SHA256(destination || 0 || contract_id || timestamp_1)
  Signature_1 = sign(Message_1, private_key)
  Submit: execute_sweep(destination, Signature_1)
  ✓ Verification succeeds
  → Nonce incremented to 1

Attempt to reuse Signature_1:
  Same message hash cannot be reconstructed because nonce = 1
  Message_1' = SHA256(destination || 1 || contract_id || timestamp_2)
  ≠ Message_1  (nonce changed)
  Signature_1.verify(Message_1', public_key) = False
  ✗ Verification fails
  → Replay attack prevented
```

## Error Handling Tree

```
execute_sweep() called
├─ verify_sweep_auth()
│  ├─ get_authorized_signer()
│  │  ├─ ✓ Found → continue
│  │  └─ ✗ Not set → AuthorizedSignerNotSet error
│  │
│  ├─ construct_sweep_message()
│  │  ├─ ✓ Message created → continue
│  │  └─ ✗ Error → Return error
│  │
│  └─ ed25519_verify()
│     ├─ ✓ Signature valid → continue
│     └─ ✗ Signature invalid → SignatureVerificationFailed error
│
├─ ✓ Authorization succeeded
│  ├─ increment_nonce() → Nonce incremented
│  ├─ Call ephemeral account sweep()
│  ├─ Execute token transfer
│  ├─ Emit event
│  └─ Return Ok(())
│
└─ ✗ Authorization failed
   └─ Return error (no state changes)
```

## Data Flow: Message Construction

```
Input Parameters:
  destination: Address
  nonce: u64
  contract_id: Address
  timestamp: u64

Processing:
  Step 1: destination.to_xdr()
          → variable-length XDR bytes
          └─ Example: [0x47, 0xBC, ..., 0xA1]

  Step 2: nonce.to_be_bytes()
          → 8 bytes (big-endian)
          └─ Example: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01]

  Step 3: contract_id.to_xdr()
          → variable-length XDR bytes
          └─ Example: [0x47, 0xBC, ..., 0xF2]

  Step 4: timestamp.to_be_bytes()
          → 8 bytes (big-endian)
          └─ Example: [0x00, 0x00, 0x00, 0x01, 0x7A, 0x5B, 0x3C, 0xD1]

Concatenation:
  message = Step1 || Step2 || Step3 || Step4
          = [0x47, 0xBC, ..., 0xA1, 0x00, ..., 0x01, 0x47, 0xBC, ..., 0xF2, 0x00, ..., 0xD1]

Hashing:
  SHA256(message) → 32-byte digest
          Example: [0x8A, 0x2C, ..., 0x7F]

Signing (Off-chain only):
  Ed25519.sign(digest, private_key) → 64-byte signature
          Example: [0x12, 0x34, ..., 0xAB]
```

## State Transitions

```
┌─────────────┐
│ Uninitialized
│ (No authorized signer)
└────────┬────┘
         │ initialize(authorized_signer)
         ↓
┌─────────────────────────────────────────┐
│ Initialized                              │
│ ├─ Authorized Signer: set              │
│ └─ Sweep Nonce: 0                      │
└────────┬────────────────────────────────┘
         │ execute_sweep(..., signature_0)
         ├─ ✓ Signature verifies
         ↓
┌─────────────────────────────────────────┐
│ After First Sweep                        │
│ ├─ Authorized Signer: unchanged         │
│ └─ Sweep Nonce: 1                       │
└────────┬────────────────────────────────┘
         │ execute_sweep(..., signature_0)
         ├─ ✗ Signature invalid (nonce changed)
         ↓
┌─────────────────────────────────────────┐
│ Rejected (Replay Attack Prevented)       │
│ └─ State unchanged, no error side-effects
└─────────────────────────────────────────┘
```

## Key Security Properties

```
┌──────────────────────────────────────────────────────────┐
│ Authentication                                           │
│ ├─ Only private key holder can create valid signature    │
│ ├─ Message is bound to signature                         │
│ └─ Cryptographic proof of authorization                  │
└──────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────┐
│ Integrity                                                │
│ ├─ Any message modification invalidates signature        │
│ ├─ Recipients can verify message wasn't altered          │
│ └─ Protects against man-in-the-middle attacks            │
└──────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────┐
│ Non-repudiation                                          │
│ ├─ Signer cannot deny having signed                      │
│ ├─ Signature mathematically proves signing              │
│ └─ Audit trail for compliance                            │
└──────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────┐
│ Replay Attack Prevention                                 │
│ ├─ Nonce makes each sweep signature unique               │
│ ├─ Old signatures become invalid after nonce increment   │
│ └─ Prevents using same signature twice                   │
└──────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────┐
│ Contract Binding                                         │
│ ├─ Signature tied to specific contract deployment        │
│ ├─ Cannot reuse signature on different contract          │
│ └─ Prevents cross-deployment attacks                     │
└──────────────────────────────────────────────────────────┘
```
