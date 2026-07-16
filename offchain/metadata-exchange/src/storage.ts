import { TravelRulePayload } from './types';

/**
 * In-memory storage for demonstration purposes.
 * Production systems should use a persistent database with encryption at rest.
 * 
 * WARNING: This storage is NOT production-ready:
 * - Data is lost on restart
 * - No encryption at rest
 * - No access controls
 * - No audit logging
 */

interface StoredPayload {
  payload: TravelRulePayload;
  hash: string;
  exchange_id: string;
  timestamp: Date;
  sender_endpoint: string;
}

// Maps hash -> stored payload
const payloadStore = new Map<string, StoredPayload>();

// Maps exchange_id -> hash
const exchangeIdIndex = new Map<string, string>();

/**
 * Store a received payload.
 * 
 * @param hash - SHA-256 hash of the payload
 * @param payload - The decrypted payload
 * @param exchange_id - Unique exchange identifier
 * @param sender_endpoint - The sending VASP's endpoint
 */
export function storePayload(
  hash: string,
  payload: TravelRulePayload,
  exchange_id: string,
  sender_endpoint: string
): void {
  // DO NOT log the payload in production (contains PII)
  console.log(`Storing payload with hash ${hash.substring(0, 8)}...`);
  
  const stored: StoredPayload = {
    payload,
    hash,
    exchange_id,
    timestamp: new Date(),
    sender_endpoint,
  };
  
  payloadStore.set(hash, stored);
  exchangeIdIndex.set(exchange_id, hash);
}

/**
 * Check if a payload with the given hash exists.
 * 
 * @param hash - SHA-256 hash to look up
 * @returns Whether a payload with this hash exists
 */
export function hasPayload(hash: string): boolean {
  return payloadStore.has(hash);
}

/**
 * Get a stored payload by its hash.
 * 
 * @param hash - SHA-256 hash to look up
 * @returns The stored payload data, or undefined if not found
 */
export function getPayload(hash: string): StoredPayload | undefined {
  return payloadStore.get(hash);
}

/**
 * Get a stored payload by its exchange ID.
 * 
 * @param exchange_id - Exchange identifier
 * @returns The stored payload data, or undefined if not found
 */
export function getPayloadByExchangeId(exchange_id: string): StoredPayload | undefined {
  const hash = exchangeIdIndex.get(exchange_id);
  if (!hash) return undefined;
  return payloadStore.get(hash);
}

/**
 * Clear all stored payloads (for testing).
 */
export function clearAll(): void {
  payloadStore.clear();
  exchangeIdIndex.clear();
}
