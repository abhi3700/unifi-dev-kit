import { argv } from "node:process";

import { client, readChain, required } from "../shared.ts";

const [, , userId, chain] = argv;
console.log(await client().getUserWalletAddress(required(userId, "a user ID"), readChain(chain)));
