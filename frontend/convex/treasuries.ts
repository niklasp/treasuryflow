import { query, mutation } from "./_generated/server";
import { v } from "convex/values";
import { getAuthUserId } from "@convex-dev/auth/server";

export const list = query({
  args: {},
  handler: async (ctx) => {
    // DEPRECATED: This query returns all treasuries without filtering.
    // Use listByOwner instead to filter by the connected account's address.
    // This is kept for backward compatibility but should not be used in production.
    throw new Error(
      "Direct list query is deprecated. Use listByOwner with owner parameter instead."
    );
  },
});

export const listByOwner = query({
  args: { owner: v.string() },
  handler: async (ctx, args) => {
    return await ctx.db
      .query("treasuries")
      .filter((q) => q.eq(q.field("owner"), args.owner))
      .collect();
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

export const getByContractAddress = query({
  args: { contractAddress: v.string() },
  handler: async (ctx, args) => {
    const treasury = await ctx.db
      .query("treasuries")
      .filter((q) => q.eq(q.field("contractAddress"), args.contractAddress))
      .first();

    if (!treasury) {
      throw new Error("Treasury not found");
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
    ss58Address: v.string(),
    network: v.optional(v.string()),
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
