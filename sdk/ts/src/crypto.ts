import { createHash } from 'crypto';
import { TravelRulePayload } from './types';

/**
 * Compute a canonical SHA-256 hash of a travel rule payload.
 * 
 * This hash is used as the `metadata_hash` parameter in the
 * `compliant_transfer` contract call. The canonicalization ensures
 * the same payload data always produces the same hash.
 * 
 * @param payload - The travel rule payload to hash
 * @returns Hex-encoded SHA-256 hash (64 characters)
 */
export function hashPayload(payload: TravelRulePayload): string {
  const canonical = canonicalizeJSON(payload);
  const hash = createHash('sha256').update(canonical, 'utf8').digest('hex');
  return hash;
}

/**
 * Convert a payload to canonical JSON format.
 * Keys are sorted alphabetically and no extra whitespace is included.
 * 
 * @internal
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
 * Convert a hex hash string to a Buffer suitable for Soroban contract calls.
 * 
 * @param hexHash - 64-character hex string from hashPayload
 * @returns Buffer representation
 */
export function hexToBuffer(hexHash: string): Buffer {
  if (hexHash.length !== 64) {
    throw new Error('Hash must be 64 hex characters (32 bytes)');
  }
  return Buffer.from(hexHash, 'hex');
}
