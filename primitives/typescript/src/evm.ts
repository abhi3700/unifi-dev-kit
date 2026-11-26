import { isAddress } from "viem/utils";

/** Checks address length and hex encoding. The `0x` prefix is optional. */
export function isEvmAddress(value: string): boolean {
  const address = value.startsWith("0x") ? value : `0x${value}`;
  return isAddress(address, { strict: false });
}
