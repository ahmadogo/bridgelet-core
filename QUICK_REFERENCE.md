# Quick Reference: Sweep Authorization

## Quick Start

### 1. Initialize Contract
```rust
// Initialize with authorized signer public key
let authorized_signer: BytesN<32> = /* 32-byte Ed25519 public key */;
controller.initialize(&authorized_signer)?;
```

### 2. Generate Signature (Off-Chain)
```
message = SHA256(
    destination_address.to_xdr() ||
    nonce.to_be_bytes() ||           // Current nonce from contract
    contract_id.to_xdr() ||
    timestamp.to_be_bytes()          // Current time
)
signature = ed25519_sign(message, private_key)  // 64-byte signature
```

### 3. Execute Sweep
```rust
controller.execute_sweep(
    &ephemeral_account,
    &destination,
    &signature  // 64-byte Ed25519 signature
)?;
```

## Message Format (Exact)

| Component | Size | Encoding |
|-----------|------|----------|
| destination | variable | XDR bytes |
| nonce | 8 bytes | big-endian u64 |
| contract_id | variable | XDR bytes |
| timestamp | 8 bytes | big-endian u64 |
| **Hash** | **32 bytes** | **SHA256** |

## Error Codes

| Error | Meaning | Solution |
|-------|---------|----------|
| AuthorizedSignerNotSet | Contract not initialized | Call initialize() first |
| SignatureVerificationFailed | Signature invalid | Verify message construction |
| InvalidNonce | Nonce mismatch | Get current nonce from contract |

## Key Points

✓ **Replay Prevention**: Nonce increments after each sweep
✓ **Contract Binding**: Signature tied to specific contract deployment
✓ **Ed25519**: Industry-standard elliptic curve signature scheme
✓ **Non-Repudiation**: Only holder of private key can generate signatures

## Testing

Generate test keypair:
```bash
openssl genpkey -algorithm ed25519 -out private.pem
openssl pkey -outform DER -pubout -in private.pem | tail -c 32 | xxd -p
```

See `SIGNATURE_FORMAT.md` for full implementation examples.
