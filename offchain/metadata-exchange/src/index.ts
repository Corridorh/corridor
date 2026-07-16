import express, { Request, Response } from 'express';
import cors from 'cors';
import { v4 as uuidv4 } from 'uuid';
import { hashPayload, encryptPayload, decryptPayload } from './crypto';
import { storePayload, hasPayload, getPayload } from './storage';
import {
  TravelRulePayload,
  InitiateRequest,
  InitiateResponse,
  ReceiveRequest,
  VerifyResponse,
} from './types';

const app = express();
const PORT = process.env.PORT || 3000;

app.use(cors());
app.use(express.json());

/**
 * POST /v1/travel-rule/initiate
 * 
 * Initiates a travel rule metadata exchange with a counterparty VASP.
 * 
 * Flow:
 * 1. Receives the originator/beneficiary payload
 * 2. Computes the canonical SHA-256 hash
 * 3. Encrypts the payload
 * 4. POSTs encrypted payload to counterparty endpoint
 * 5. Returns the hash (for use in on-chain compliant_transfer call)
 */
app.post('/v1/travel-rule/initiate', async (req: Request, res: Response) => {
  try {
    const { payload, counterparty_endpoint }: InitiateRequest = req.body;
    
    // Validate payload structure
    if (!payload || !payload.originator || !payload.beneficiary) {
      return res.status(400).json({ error: 'Invalid payload structure' });
    }
    
    if (!counterparty_endpoint) {
      return res.status(400).json({ error: 'counterparty_endpoint is required' });
    }
    
    // Generate exchange ID
    const exchange_id = uuidv4();
    
    // Compute hash
    const metadata_hash = hashPayload(payload);
    
    // Encrypt payload (stub - in production, use counterparty's public key)
    const encrypted_payload = encryptPayload(payload);
    
    // Send to counterparty (stub - in production, use mutual TLS)
    // For demo purposes, we'll skip the actual HTTP call
    console.log(`Would POST to ${counterparty_endpoint} with exchange_id ${exchange_id}`);
    console.log(`Hash: ${metadata_hash}`);
    
    // In a real implementation:
    // await axios.post(`${counterparty_endpoint}/v1/travel-rule/receive`, {
    //   encrypted_payload,
    //   sender_vasp_endpoint: process.env.OUR_ENDPOINT,
    //   exchange_id,
    // });
    
    const response: InitiateResponse = {
      metadata_hash,
      exchange_id,
    };
    
    res.json(response);
  } catch (error) {
    console.error('Error initiating travel rule exchange:', error);
    res.status(500).json({ error: 'Internal server error' });
  }
});

/**
 * POST /v1/travel-rule/receive
 * 
 * Receives an encrypted travel rule payload from a counterparty VASP.
 * 
 * Flow:
 * 1. Receives encrypted payload
 * 2. Decrypts using our private key
 * 3. Computes hash to verify integrity
 * 4. Stores payload keyed by hash
 * 5. Returns 200 OK
 */
app.post('/v1/travel-rule/receive', async (req: Request, res: Response) => {
  try {
    const { encrypted_payload, sender_vasp_endpoint, exchange_id }: ReceiveRequest = req.body;
    
    if (!encrypted_payload || !sender_vasp_endpoint || !exchange_id) {
      return res.status(400).json({ error: 'Missing required fields' });
    }
    
    // Decrypt payload (stub - in production, use our private key from KMS)
    const payload: TravelRulePayload = decryptPayload(encrypted_payload);
    
    // Compute hash to verify and store
    const hash = hashPayload(payload);
    
    // Store the payload (DO NOT log it - contains PII)
    storePayload(hash, payload, exchange_id, sender_vasp_endpoint);
    
    res.json({ 
      success: true,
      hash,
      exchange_id,
    });
  } catch (error) {
    console.error('Error receiving travel rule payload:', error);
    res.status(500).json({ error: 'Internal server error' });
  }
});

/**
 * GET /v1/travel-rule/verify/:hash
 * 
 * Verifies whether a payload matching the given hash has been received.
 * 
 * Used by the sending VASP to confirm the counterparty received the metadata
 * before submitting the on-chain transaction.
 */
app.get('/v1/travel-rule/verify/:hash', (req: Request, res: Response) => {
  try {
    const { hash } = req.params;
    
    if (!hash || hash.length !== 64) {
      return res.status(400).json({ error: 'Invalid hash format (must be 64 hex characters)' });
    }
    
    const exists = hasPayload(hash);
    const payload = exists ? getPayload(hash) : undefined;
    
    const response: VerifyResponse = {
      exists,
      exchange_id: payload?.exchange_id,
    };
    
    res.json(response);
  } catch (error) {
    console.error('Error verifying hash:', error);
    res.status(500).json({ error: 'Internal server error' });
  }
});

/**
 * GET /health
 * 
 * Health check endpoint
 */
app.get('/health', (req: Request, res: Response) => {
  res.json({ status: 'ok', service: 'travel-rule-metadata-exchange' });
});

app.listen(PORT, () => {
  console.log(`🚀 Travel Rule Metadata Exchange service running on port ${PORT}`);
  console.log(`⚠️  WARNING: This is a reference implementation with stub encryption`);
  console.log(`⚠️  DO NOT use in production without proper KMS/HSM integration`);
});
