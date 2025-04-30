"use client"

import { DashboardClient } from "@/components/dashboard-client"

// Sample data for the dashboard
const treasuryData = {
  totalAmount: 125000,
  nextPayoutDays: 7,
  pendingPayouts: 3,
  chartData: [
    { name: "Jan", amount: 65000 },
    { name: "Feb", amount: 78000 },
    { name: "Mar", amount: 92000 },
    { name: "Apr", amount: 85000 },
    { name: "May", amount: 110000 },
    { name: "Jun", amount: 125000 },
  ],
  payouts: [
    { id: 1, recipient: "Vendor A", amount: 12500, unit: "USD", date: "2025-04-10" },
    { id: 2, recipient: "Contractor B", amount: 8750, unit: "USD", date: "2025-04-05" },
    { id: 3, recipient: "Supplier C", amount: 15000, unit: "USD", date: "2025-03-28" },
    { id: 4, recipient: "Partner D", amount: 6200, unit: "USD", date: "2025-03-15" },
    { id: 5, recipient: "Agency E", amount: 9800, unit: "USD", date: "2025-03-01" },
  ],
}

export default function DashboardPage() {
  return <DashboardClient treasuryData={treasuryData} />
}
