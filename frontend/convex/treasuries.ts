import { query, mutation } from "./_generated/server";
import { v } from "convex/values";
import { getAuthUserId } from "@convex-dev/auth/server";

export const list = query({
  args: {},
  handler: async (ctx) => {
    // todo get the account id from the user
    return await ctx.db.query("treasuries").collect();
  },
});

export const get = query({
  args: { treasuryId: v.id("treasuries") },
  handler: async (ctx, args) => {
    const userId = await getAuthUserId(ctx);
    if (!userId) {
      throw new Error("Not authenticated");
    }

    const treasury = await ctx.db.get(args.treasuryId);
    if (!treasury || treasury.owner !== userId) {
      throw new Error("Treasury not found or access denied");
    }

    return treasury;
  },
});

export const create = mutation({
  args: {
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
  },
  handler: async (ctx, args) => {
    return await ctx.db.insert("treasuries", {
      ...args,
    });
  },
});
