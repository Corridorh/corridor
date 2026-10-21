import { createHash, randomBytes, createCipheriv, createDecipheriv } from 'crypto';
import { TravelRulePayload } from './types';

/**
 * Compute a canonical SHA-256 hash of a travel rule payload.
 * 
 * The canonicalization ensures that the same payload data always produces
 * the same hash, regardless of key ordering or formatting in the JSON.
 * 
 * @param payload - The travel rule payload to hash
 * @returns Hex-encoded SHA-256 hash (64 characters)
 */
export function hashPayload(payload: TravelRulePayload): string {
  // Canonicalize by sorting keys and removing whitespace
  const canonical = canonicalizeJSON(payload);
  const hash = createHash('sha256').update(canonical, 'utf8').digest('hex');
  return hash;
}

/**
 * Convert a payload to canonical JSON format.
 * Keys are sorted alphabetically and no extra whitespace is included.
 */
function canonicalizeJSON(obj: any): string {
  if (obj === null || obj === undefined) {
    return JSON.stringify(obj);
  }
  
  if (typeof obj !== 'object') {
    return JSON.stringify(obj);
  }
  
  if (Array.isArray(obj)) {
    return '[' + obj.map(canonicalizeJSON).join(',') + ']';
  }
  
  const keys = Object.keys(obj).sort();
  const pairs = keys.map(key => `"${key}":${canonicalizeJSON(obj[key])}`);
  return '{' + pairs.join(',') + '}';
}

/**
 * Encrypt a payload for transmission to a counterparty VASP.
 * 
 * WARNING: This is a STUB implementation for demonstration purposes only.
 * Production systems MUST use:
 * - Proper asymmetric encryption (e.g., recipient's public key)
 * - Hardware Security Module (HSM) or Key Management Service (KMS)
 * - Mutual TLS between VASPs
 * - Signed requests with non-repudiation
 * 
 * @param payload - The payload to encrypt
 * @param recipientPublicKey - Recipient's public key (STUB - not actually used)
 * @returns Base64-encoded encrypted payload
 */
export function encryptPayload(payload: TravelRulePayload, recipientPublicKey?: string): string {
  // TODO: Replace with production KMS/HSM integration
  // This stub uses symmetric AES-256-GCM for demonstration only
  
  console.warn('⚠️  STUB ENCRYPTION - NOT PRODUCTION READY');
  
  const payloadString = JSON.stringify(payload);
  
  // Generate a random key and IV (this should come from KMS in production)
  const key = randomBytes(32); // 256-bit key
  const iv = randomBytes(16);  // 128-bit IV
  
  const cipher = createCipheriv('aes-256-gcm', key, iv);
  let encrypted = cipher.update(payloadString, 'utf8', 'base64');
  encrypted += cipher.final('base64');
  const authTag = cipher.getAuthTag().toString('base64');
  
  // In production, the key would be encrypted with the recipient's public key
  // Here we just bundle everything together (INSECURE)
  const bundle = {
    encrypted,
    authTag,
    key: key.toString('base64'),
    iv: iv.toString('base64'),
  };
  
  return Buffer.from(JSON.stringify(bundle)).toString('base64');
}

/**
 * Decrypt a received payload.
 * 
 * WARNING: This is a STUB implementation. Production systems need real key management.
 * 
 * @param encryptedPayload - Base64-encoded encrypted payload
 * @param privateKey - Recipient's private key (STUB - not actually used)
 * @returns Decrypted payload object
 */
export function decryptPayload(encryptedPayload: string, privateKey?: string): TravelRulePayload {
  // TODO: Replace with production KMS/HSM integration
  
  console.warn('⚠️  STUB DECRYPTION - NOT PRODUCTION READY');
  
  const bundleString = Buffer.from(encryptedPayload, 'base64').toString('utf8');
  const bundle = JSON.parse(bundleString);
  
  const key = Buffer.from(bundle.key, 'base64');
  const iv = Buffer.from(bundle.iv, 'base64');
  const authTag = Buffer.from(bundle.authTag, 'base64');
  
  const decipher = createDecipheriv('aes-256-gcm', key, iv);
  decipher.setAuthTag(authTag);
  
  let decrypted = decipher.update(bundle.encrypted, 'base64', 'utf8');
  decrypted += decipher.final('utf8');
  
  return JSON.parse(decrypted);
}

/**
 * Convert a hex hash to a Soroban BytesN<32> compatible format.
 * 
 * @param hexHash - 64-character hex string
 * @returns Buffer representation
 */
export function hexToBytes32(hexHash: string): Buffer {
  if (!/^[0-9a-fA-F]{64}$/.test(hexHash)) {
    throw new Error('Hash must be 64 hex characters (32 bytes)');
  }
  return Buffer.from(hexHash, 'hex');
}
