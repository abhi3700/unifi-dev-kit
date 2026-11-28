import { argv } from "node:process";

import { client, required } from "../shared.ts";

const owner = required(argv[2], "the API key owner's user ID");
const { metadata } = await client().getApiAccount(owner);
console.log({ plan: metadata.plan, remainingCredits: metadata.credits });
