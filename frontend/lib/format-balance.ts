/**
 * Format a balance value for display
 * @param balance - Balance in planck (smallest unit)
 * @param decimals - Number of decimal places (default: 12 for DOT)
 * @param symbol - Token symbol (default: DOT)
 * @returns Formatted balance string
 */
export function formatBalance(
  balance: bigint | number | string,
  decimals: number = 12,
  symbol: string = "DOT"
): string {
  const balanceBigInt = typeof balance === "bigint" ? balance : BigInt(balance);

  // Convert from planck to token units
  const divisor = BigInt(10 ** decimals);
  const wholePart = balanceBigInt / divisor;
  const fractionalPart = balanceBigInt % divisor;

  // Format the fractional part with leading zeros
  const fractionalStr = fractionalPart.toString().padStart(decimals, "0");

  // Remove trailing zeros from fractional part
  const trimmedFractional = fractionalStr.replace(/0+$/, "");

  // Combine whole and fractional parts
  const formattedValue = trimmedFractional
    ? `${wholePart.toString()}.${trimmedFractional}`
    : wholePart.toString();

  return `${formattedValue} ${symbol}`;
}

/**
 * Format balance with compact notation for large numbers
 */
export function formatBalanceCompact(
  balance: bigint | number | string,
  decimals: number = 12,
  symbol: string = "DOT"
): string {
  const balanceBigInt = typeof balance === "bigint" ? balance : BigInt(balance);
  const divisor = BigInt(10 ** decimals);
  const tokenAmount = Number(balanceBigInt) / Number(divisor);

  // Use compact notation for large numbers
  const formatter = new Intl.NumberFormat("en-US", {
    notation: "compact",
    maximumFractionDigits: 2,
  });

  return `${formatter.format(tokenAmount)} ${symbol}`;
}

/**
 * Format balance for the specific case used in account balance component
 */
export function formatBalanceObject({
  balance,
  decimals = 12,
  symbol = "DOT",
}: {
  balance: bigint;
  decimals?: number;
  symbol?: string;
}) {
  return formatBalance(balance, decimals, symbol);
}
