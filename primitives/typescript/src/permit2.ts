import { maxUint256 } from "viem";
import type { Address } from "viem";

export interface DomainPermitForSignature {
  domain: {
    name?: string;
    chainId: string;
    verifyingContract: Address;
  };
  permit: {
    permitted: { token: Address; amount: string }[];
    spender: Address;
    nonce: string;
    deadline: string;
  };
}

const permit2Types = {
  TokenPermissions: [
    { name: "token", type: "address" },
    { name: "amount", type: "uint256" },
  ],
  PermitBatchTransferFrom: [
    { name: "permitted", type: "TokenPermissions[]" },
    { name: "spender", type: "address" },
    { name: "nonce", type: "uint256" },
    { name: "deadline", type: "uint256" },
  ],
} as const;

function toUint256(value: string): bigint {
  if (!/^(?:0x[\da-f]+|\d+)$/i.test(value)) throw new RangeError("Invalid Permit2 uint256");
  const integer = BigInt(value);
  if (integer > maxUint256) throw new RangeError("Permit2 integer exceeds uint256");
  return integer;
}

/** Converts the server's Permit2 response to viem's signTypedData input. */
export function toPermit2TypedData({ domain, permit }: DomainPermitForSignature) {
  return {
    domain: {
      name: domain.name ?? "Permit2",
      chainId: toUint256(domain.chainId),
      verifyingContract: domain.verifyingContract,
    },
    primaryType: "PermitBatchTransferFrom",
    types: permit2Types,
    message: {
      permitted: permit.permitted.map(({ token, amount }) => ({ token, amount: toUint256(amount) })),
      spender: permit.spender,
      nonce: toUint256(permit.nonce),
      deadline: toUint256(permit.deadline),
    },
  } as const;
}
