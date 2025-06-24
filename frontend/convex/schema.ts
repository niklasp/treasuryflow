import { defineSchema, defineTable } from "convex/server";
import { v } from "convex/values";
import { authTables } from "@convex-dev/auth/server";

const applicationTables = {
  treasuries: defineTable({
    owner: v.string(),
    name: v.string(),
    description: v.optional(v.string()),
    contractAddress: v.string(),
    ss58Address: v.string(),
    currencies: v.optional(v.array(v.string())),
    payoutFrequency: v.optional(v.string()),
    treasurers: v.optional(
      v.array(
        v.object({
          name: v.string(),
          address: v.string(),
        })
      )
    ),
  }).index("by_owner", ["owner"]),
};

export default defineSchema({
  ...authTables,
  ...applicationTables,
});
