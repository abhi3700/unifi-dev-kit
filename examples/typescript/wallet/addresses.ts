import { argv } from "node:process";

import { client, required } from "../shared.ts";

console.log(await client().getUserWalletAddresses(required(argv[2], "a user ID")));
