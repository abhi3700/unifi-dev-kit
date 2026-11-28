import { argv } from "node:process";

import { client, required } from "../shared.ts";

const [, , userId, cursor] = argv;
// Pass the next cursor with the same filters to read another page.
const page = await client().getContacts(required(userId, "a user ID"), { protocol: "Evm", limit: 10, cursor });
for (const contact of page.contacts) console.log(`${contact.name}\t${contact.address}`);
if (page.next_cursor) console.log(`Next cursor ${page.next_cursor}`);
