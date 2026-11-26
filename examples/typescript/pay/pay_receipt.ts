import { argv } from "node:process";

import { client, required } from "../shared.ts";

console.log(await client().getPaymentReceipt(required(argv[2], "a receipt ID")));
