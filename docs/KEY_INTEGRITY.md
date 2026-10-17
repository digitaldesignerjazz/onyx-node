# Key Integrity — Supervisor Fingerprint Check

The Onyx node verifies the ML-KEM-1024 supervisor public key on every pulse.

## Files

- `keys/supervisor_kyber1024_public.txt` — public key (Base64 + hex)
- `keys/supervisor_kyber1024_metadata.json` — reference fingerprint `0D9FE10BE523064E`
- `scripts/verify_supervisor_key.sh` — verification script

## Behavior

1. Parse the Base64 public key from the public key file.
2. Decode to raw bytes, compute SHA3-256, take the first 16 hex chars.
3. Compare against the fingerprint in the metadata file.
4. Exit 0 on match, exit 1 on mismatch (possible key swap / tamper).

## Integration

Call from the listen-loop before emitting a pulse:

```bash
./scripts/verify_supervisor_key.sh || { echo "key mismatch"; exit 1; }
```

A mismatch should fail the pulse and alert the operator. The private 64-byte seed
is never present on the node — air-gapped only.
