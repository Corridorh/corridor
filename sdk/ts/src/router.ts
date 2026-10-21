import {
  Contract,
  SorobanRpc,
  TransactionBuilder,
  Networks,
  BASE_FEE,
  xdr,
  Address,
  nativeToScVal,
} from '@stellar/stellar-sdk';
import { CompliantTransferParams } from './types';
import { hexToBuffer } from './crypto';

/**
 * Build a transaction that invokes the `compliant_transfer` function
 * on the router contract.
 * 
 * @param params - Transaction parameters
 * @returns Promise<string> - XDR string of the prepared transaction (needs signing)
 */
export async function buildCompliantTransferTx(
  params: CompliantTransferParams
): Promise<string> {
  const {
    routerContractId,
    from,
    to,
    amount,
    metadataHash,
    networkPassphrase,
    rpcUrl,
    sourceAccount,
  } = params;

  // Create RPC server instance
  const server = new SorobanRpc.Server(rpcUrl);

  // Get source account
  const sourceAddr = sourceAccount || from;
  const account = await server.getAccount(sourceAddr);

  // Create contract instance
  const contract = new Contract(routerContractId);

  // Build contract call operation.
  //
  // Soroban encodes `Option<T>` as either `ScVal::Void` (None) or the inner
  // value itself (Some) — there is no tagged "option" ScVal.
  let metadataHashScVal: xdr.ScVal;

  if (metadataHash) {
    const hashBuffer = hexToBuffer(metadataHash);
    metadataHashScVal = nativeToScVal(hashBuffer, { type: 'bytes' });
  } else {
    metadataHashScVal = xdr.ScVal.scvVoid();
  }

  // Build the transaction
  const builtTx = new TransactionBuilder(account, {
    fee: BASE_FEE,
    networkPassphrase: networkPassphrase === 'testnet' 
      ? Networks.TESTNET 
      : networkPassphrase,
  })
    .addOperation(
      contract.call(
        'compliant_transfer',
        new Address(from).toScVal(),
        new Address(to).toScVal(),
        nativeToScVal(BigInt(amount), { type: 'i128' }),
        metadataHashScVal
      )
    )
    .setTimeout(180)
    .build();

  // Simulate the transaction
  const simulated = await server.simulateTransaction(builtTx);

  if (SorobanRpc.Api.isSimulationError(simulated)) {
    throw new Error(`Simulation failed: ${JSON.stringify(simulated)}`);
  }

  // Prepare the transaction with simulation results
  const preparedTx = SorobanRpc.assembleTransaction(builtTx, simulated).build();

  return preparedTx.toXDR();
}

/**
 * Check if an address is a registered VASP in the registry.
 * 
 * @param registryContractId - The registry contract ID
 * @param vaspAddress - The address to check
 * @param rpcUrl - Soroban RPC URL
 * @returns Promise<boolean> - Whether the address is a registered, active VASP
 */
export async function verifyVaspRegistered(
  registryContractId: string,
  vaspAddress: string,
  rpcUrl: string
): Promise<boolean> {
  const server = new SorobanRpc.Server(rpcUrl);
  const contract = new Contract(registryContractId);

  try {
    // Create a dummy source account for simulation (not used in read-only call)
    const dummySource = 'GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF';
    const account = await server.getAccount(dummySource).catch(() => {
      // If dummy account doesn't exist, create a minimal account object
      return {
        accountId: () => dummySource,
        sequenceNumber: () => '0',
        incrementSequenceNumber: () => {},
      } as any;
    });

    // Build view call
    const builtTx = new TransactionBuilder(account, {
      fee: BASE_FEE,
      networkPassphrase: Networks.TESTNET,
    })
      .addOperation(
        contract.call('is_registered', new Address(vaspAddress).toScVal())
      )
      .setTimeout(180)
      .build();

    const simulated = await server.simulateTransaction(builtTx);

    if (SorobanRpc.Api.isSimulationSuccess(simulated) && simulated.result) {
      const resultValue = simulated.result.retval;
      if (resultValue.switch().name !== 'scvBool') {
        return false;
      }
      const value = resultValue.value();
      return typeof value === 'boolean' ? value : false;
    }

    return false;
  } catch (error) {
    console.error('Error verifying VASP registration:', error);
    return false;
  }
}
