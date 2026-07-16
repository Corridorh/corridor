# Travel Rule Metadata Exchange Service

Reference implementation of an off-chain metadata exchange service for the Soroban Travel Rule Router system.

## ⚠️ Security Warning

**This is a reference implementation with stub encryption. DO NOT use in production without:**

- Proper asymmetric encryption (recipient's public key)
- Hardware Security Module (HSM) or Key Management Service (KMS)
- Mutual TLS between VASPs
- Signed requests with non-repudiation
- Access controls and audit logging
- Persistent database with encryption at rest

All encryption functions are marked with `// TODO: replace with production KMS` comments.

## Purpose

This service demonstrates how two VASPs actually exchange IVMS101-lite compliance payloads before submitting a `compliant_transfer` transaction on-chain. The key workflow is:

1. **Sending VASP** calls `/v1/travel-rule/initiate` with originator/beneficiary data
2. Service computes SHA-256 hash of the canonical payload
3. Service encrypts the payload and POSTs to the receiving VASP's endpoint
4. **Receiving VASP** receives via `/v1/travel-rule/receive`, decrypts, and stores
5. Sending VASP uses the hash in the on-chain `compliant_transfer` call
6. Either party can verify hash existence via `/v1/travel-rule/verify/:hash`

## API Endpoints

### POST `/v1/travel-rule/initiate`

Initiate a metadata exchange with a counterparty VASP.

**Request:**
```json
{
  "payload": {
    "originator": {
      "name": "Alice Smith",
      "vasp_id": "VASP-A-LEI-123",
      "account_ref": "alice-001"
    },
    "beneficiary": {
      "name": "Bob Jones",
      "vasp_id": "VASP-B-LEI-456",
      "account_ref": "bob-002"
    },
    "amount": "5000",
    "asset_code": "USDC",
    "timestamp": "2024-01-15T10:30:00Z"
  },
  "counterparty_endpoint": "https://vasp-b.example.com"
}
```

**Response:**
```json
{
  "metadata_hash": "a3f5b8c2...",
  "exchange_id": "uuid-here"
}
```

### POST `/v1/travel-rule/receive`

Receive an encrypted payload from a counterparty VASP.

**Request:**
```json
{
  "encrypted_payload": "base64-encoded-encrypted-data",
  "sender_vasp_endpoint": "https://vasp-a.example.com",
  "exchange_id": "uuid-here"
}
```

**Response:**
```json
{
  "success": true,
  "hash": "a3f5b8c2...",
  "exchange_id": "uuid-here"
}
```

### GET `/v1/travel-rule/verify/:hash`

Verify whether a payload with the given hash has been received.

**Response:**
```json
{
  "exists": true,
  "exchange_id": "uuid-here"
}
```

## Running Locally

```bash
npm install
npm run dev
```

Server runs on port 3000 (configurable via `PORT` environment variable).

## Testing

```bash
npm test
```

## Data Storage

Currently uses in-memory storage (lost on restart). Production systems should use:
- Persistent database (PostgreSQL, MongoDB, etc.)
- Encryption at rest
- Access controls
- Audit logging
- Retention policies compliant with data privacy regulations

## Next Steps for Production

1. **Replace stub encryption** with real asymmetric cryptography
2. **Integrate KMS/HSM** for key management
3. **Implement mutual TLS** for VASP-to-VASP communication
4. **Add request signing** for non-repudiation
5. **Set up persistent database** with proper schema
6. **Add authentication/authorization** for API endpoints
7. **Implement rate limiting** and DDoS protection
8. **Add comprehensive logging** (without PII in logs)
9. **Set up monitoring and alerting**
10. **Conduct security audit** before production deployment
