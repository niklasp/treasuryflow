import { defineSchema, defineTable } from "convex/server";
import { v } from "convex/values";
import { authTables } from "@convex-dev/auth/server";

const applicationTables = {
  treasuries: defineTable({
    owner: v.string(),
    name: v.string(),
    description: v.optional(v.string()),
    contractAddress: v.string(),
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

  //   payouts: defineTable({
  //     treasuryId: v.id("treasuries"),
  //     recipient: v.string(),
  //     amount: v.number(),
  //     category: v.string(),
  //     type: v.union(
  //       v.literal("one-time"),
  //       v.literal("recurring"),
  //       v.literal("vested")
  //     ),
  //     status: v.union(
  //       v.literal("active"),
  //       v.literal("paused"),
  //       v.literal("completed"),
  //       v.literal("cancelled")
  //     ),
  //     intervalSeconds: v.optional(v.number()),
  //     vestingStartTime: v.optional(v.number()),
  //     vestingDurationSeconds: v.optional(v.number()),
  //     totalVestingAmount: v.optional(v.number()),
  //     amountClaimed: v.optional(v.number()),
  //     lastExecutedAt: v.optional(v.number()),
  //     nextExecutionAt: v.optional(v.number()),
  //     createdBy: v.id("users"),
  //   })
  //     .index("by_treasury", ["treasuryId"])
  //     .index("by_next_execution", ["nextExecutionAt"])
  //     .index("by_status", ["status"]),

  //   transfers: defineTable({
  //     payoutId: v.id("payouts"),
  //     treasuryId: v.id("treasuries"),
  //     recipient: v.string(),
  //     amount: v.number(),
  //     category: v.string(),
  //     status: v.union(
  //       v.literal("pending"),
  //       v.literal("completed"),
  //       v.literal("failed"),
  //       v.literal("superseded")
  //     ),
  //     transactionHash: v.optional(v.string()),
  //     errorMessage: v.optional(v.string()),
  //     executedAt: v.number(),
  //   })
  //     .index("by_treasury", ["treasuryId"])
  //     .index("by_payout", ["payoutId"])
  //     .index("by_status", ["status"]),
};

export default defineSchema({
  ...authTables,
  ...applicationTables,
});
