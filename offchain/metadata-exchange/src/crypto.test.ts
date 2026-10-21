import { hashPayload, encryptPayload, decryptPayload, hexToBytes32 } from './crypto';
import { TravelRulePayload } from './types';

const payload: TravelRulePayload = {
  originator: { name: 'Alice', vasp_id: 'VASP-A', account_ref: 'acct-1' },
  beneficiary: { name: 'Bob', vasp_id: 'VASP-B', account_ref: 'acct-2' },
  amount: '5000',
  asset_code: 'USDC',
  timestamp: '2026-01-01T00:00:00.000Z',
};

describe('hashPayload', () => {
  it('is deterministic and order-independent', () => {
    const reordered = {
      timestamp: payload.timestamp,
      asset_code: payload.asset_code,
      amount: payload.amount,
      beneficiary: { ...payload.beneficiary },
      originator: { ...payload.originator },
    } as TravelRulePayload;

    expect(hashPayload(payload)).toMatch(/^[0-9a-f]{64}$/);
    expect(hashPayload(reordered)).toBe(hashPayload(payload));
  });
});

describe('encryptPayload / decryptPayload', () => {
  // The reference implementation logs a warning on every call; keep test output
  // focused on failures.
  beforeAll(() => {
    jest.spyOn(console, 'warn').mockImplementation(() => undefined);
  });

  afterAll(() => {
    jest.restoreAllMocks();
  });

  it('round-trips a payload', () => {
    const encrypted = encryptPayload(payload);
    expect(decryptPayload(encrypted)).toEqual(payload);
  });

  it('produces a fresh ciphertext each time', () => {
    expect(encryptPayload(payload)).not.toBe(encryptPayload(payload));
  });

  it('rejects a tampered ciphertext', () => {
    const bundle = JSON.parse(Buffer.from(encryptPayload(payload), 'base64').toString('utf8'));
    bundle.encrypted = Buffer.from('tampered').toString('base64');
    const tampered = Buffer.from(JSON.stringify(bundle)).toString('base64');

    expect(() => decryptPayload(tampered)).toThrow();
  });
});

describe('hexToBytes32', () => {
  it('decodes 64 hex characters', () => {
    expect(hexToBytes32('11'.repeat(32))).toHaveLength(32);
  });

  it('rejects the wrong length', () => {
    expect(() => hexToBytes32('11')).toThrow(/64 hex characters/);
  });

  it('rejects non-hex characters', () => {
    expect(() => hexToBytes32('zz'.repeat(32))).toThrow(/64 hex characters/);
  });
});
