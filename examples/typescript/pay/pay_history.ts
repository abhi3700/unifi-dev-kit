import { argv } from "node:process";

import { client, required } from "../shared.ts";

const [, , userId, cursor] = argv;
console.log(await client().getPaymentHistory(
  required(userId, "a user ID"),
  true,
  cursor === undefined,
  { limit: 10, cursor, next_or_previous: cursor === undefined ? undefined : true },
));
