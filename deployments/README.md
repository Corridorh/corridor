# Deployment records

Every first-party deployment must publish a machine-readable record here. The
record is the source of truth for contract IDs, the commit that produced the
WASM, and the configuration used to initialize the contracts.

## Schema

```json
{
  "network": "testnet",
  "commit": "<40-character git SHA>",
  "deployed_at": "<ISO-8601 timestamp>",
  "admin": "<Stellar address>",
  "contracts": {
    "vasp_registry": "<contract id>",
    "router": "<contract id>",
    "token": "<contract id>"
  },
  "router": {
    "threshold": "1000000000"
  },
  "vasps": [
    {
      "address": "<Stellar address>",
      "endpoint_url": "https://..."
    }
  ]
}
```

Rules:

- `commit` must be a full SHA that exists in this repository.
- Contract IDs use the `C...` StrKey form.
- `threshold` is a decimal string in the token's smallest unit (i128).
- Do not include secrets, private keys, or personal data.
- A new deployment gets a new file; do not rewrite history.

See [`testnet.example.json`](./testnet.example.json) for a template.
