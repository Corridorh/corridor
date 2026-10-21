import { createHash } from 'crypto';
import { hashPayload, hexToBuffer } from './crypto';
import { TravelRulePayload } from './types';

const payload: TravelRulePayload = {
  originator: { name: 'Alice', vasp_id: 'VASP-A', account_ref: 'acct-1' },
  beneficiary: { name: 'Bob', vasp_id: 'VASP-B', account_ref: 'acct-2' },
  amount: '5000',
  asset_code: 'USDC',
  timestamp: '2026-01-01T00:00:00.000Z',
};

describe('hashPayload', () => {
  it('returns a 64-character lowercase hex SHA-256 digest', () => {
    expect(hashPayload(payload)).toMatch(/^[0-9a-f]{64}$/);
  });

  it('is deterministic', () => {
    expect(hashPayload(payload)).toBe(hashPayload(payload));
  });

  it('is independent of property insertion order', () => {
    const reordered: TravelRulePayload = {
      timestamp: '2026-01-01T00:00:00.000Z',
      asset_code: 'USDC',
      amount: '5000',
      beneficiary: { account_ref: 'acct-2', vasp_id: 'VASP-B', name: 'Bob' },
      originator: { account_ref: 'acct-1', vasp_id: 'VASP-A', name: 'Alice' },
    };

    expect(hashPayload(reordered)).toBe(hashPayload(payload));
  });

  it('matches an independently canonicalized digest', () => {
    // Keys sorted at every level, no whitespace.
    const canonical =
      '{"amount":"5000","asset_code":"USDC",' +
      '"beneficiary":{"account_ref":"acct-2","name":"Bob","vasp_id":"VASP-B"},' +
      '"originator":{"account_ref":"acct-1","name":"Alice","vasp_id":"VASP-A"},' +
      '"timestamp":"2026-01-01T00:00:00.000Z"}';
    const expected = createHash('sha256').update(canonical, 'utf8').digest('hex');

    expect(hashPayload(payload)).toBe(expected);
  });

  it('changes when any committed field changes', () => {
    const tampered: TravelRulePayload = {
      ...payload,
      amount: '5001',
    };
    expect(hashPayload(tampered)).not.toBe(hashPayload(payload));
  });
});

describe('hexToBuffer', () => {
  it('decodes a 64-character hex string into 32 bytes', () => {
    const buffer = hexToBuffer('00'.repeat(31) + 'ff');
    expect(buffer).toHaveLength(32);
    expect(buffer[31]).toBe(0xff);
  });

  it('rejects strings that are not 32 bytes of hex', () => {
    expect(() => hexToBuffer('abcd')).toThrow(/64 hex characters/);
    expect(() => hexToBuffer('zz'.repeat(32))).toThrow(/64 hex characters/);
  });
});
