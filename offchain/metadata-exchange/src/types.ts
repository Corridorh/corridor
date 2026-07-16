/**
 * IVMS101-lite payload structure for MVP.
 * 
 * This is a simplified version of the full IVMS101 schema used by
 * real-world travel rule protocols. Production systems should implement
 * the complete IVMS101 standard.
 */
export interface TravelRulePayload {
  originator: {
    /** Full name of the originator */
    name: string;
    /** VASP identifier (e.g., LEI, registration number) */
    vasp_id: string;
    /** Account reference at the originating VASP */
    account_ref: string;
  };
  beneficiary: {
    /** Full name of the beneficiary */
    name: string;
    /** VASP identifier (e.g., LEI, registration number) */
    vasp_id: string;
    /** Account reference at the beneficiary VASP */
    account_ref: string;
  };
  /** Transfer amount as string (to match Soroban i128) */
  amount: string;
  /** Asset code (e.g., "USDC", "XLM") */
  asset_code: string;
  /** ISO8601 timestamp */
  timestamp: string;
}

export interface InitiateRequest {
  payload: TravelRulePayload;
  /** The counterparty VASP's endpoint URL */
  counterparty_endpoint: string;
}

export interface InitiateResponse {
  /** SHA-256 hash of the canonicalized payload (hex encoded) */
  metadata_hash: string;
  /** Unique identifier for this exchange */
  exchange_id: string;
}

export interface ReceiveRequest {
  /** Encrypted payload (base64 encoded) */
  encrypted_payload: string;
  /** Endpoint of the sending VASP */
  sender_vasp_endpoint: string;
  /** Exchange identifier from the initiating request */
  exchange_id: string;
}

export interface VerifyResponse {
  /** Whether a payload matching this hash exists */
  exists: boolean;
  /** The exchange ID if found */
  exchange_id?: string;
}
