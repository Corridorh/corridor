/**
 * IVMS101-lite payload structure.
 * 
 * Simplified travel rule metadata for the MVP implementation.
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

/**
 * Parameters for building a compliant transfer transaction.
 */
export interface CompliantTransferParams {
  /** The router contract ID (C...) */
  routerContractId: string;
  /** Sender's Stellar address (G...) */
  from: string;
  /** Recipient's Stellar address (G...) */
  to: string;
  /** Transfer amount as string (matches Soroban i128) */
  amount: string;
  /** Optional metadata hash (required for above-threshold transfers) */
  metadataHash?: string;
  /** Network passphrase (e.g., "Test SDF Network ; September 2015") */
  networkPassphrase: string;
  /** Soroban RPC URL */
  rpcUrl: string;
  /** Source account for the transaction (defaults to 'from' if not provided) */
  sourceAccount?: string;
}
