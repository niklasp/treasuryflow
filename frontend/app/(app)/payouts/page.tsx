import { PayoutsContent } from "@/components/payouts-content";

// Sample data for payouts
const payoutsData = {
  totalPayouts: 47,
  totalAmount: 285000,
  pendingAmount: 28500,
  payouts: [
    {
      id: 1,
      recipient: "Vendor A",
      amount: 12500,
      unit: "USD",
      date: "2025-04-10",
      status: "pending",
      description: "Monthly service fee",
    },
    {
      id: 2,
      recipient: "Contractor B",
      amount: 8750,
      unit: "USD",
      date: "2025-04-05",
      status: "completed",
      description: "Development work",
    },
    {
      id: 3,
      recipient: "Supplier C",
      amount: 15000,
      unit: "USD",
      date: "2025-03-28",
      status: "completed",
      description: "Equipment purchase",
    },
    {
      id: 4,
      recipient: "Partner D",
      amount: 6200,
      unit: "USD",
      date: "2025-03-15",
      status: "completed",
      description: "Consulting services",
    },
    {
      id: 5,
      recipient: "Agency E",
      amount: 9800,
      unit: "USD",
      date: "2025-03-01",
      status: "completed",
      description: "Marketing campaign",
    },
    {
      id: 6,
      recipient: "Freelancer F",
      amount: 3200,
      unit: "USD",
      date: "2025-02-28",
      status: "completed",
      description: "Design work",
    },
    {
      id: 7,
      recipient: "Vendor G",
      amount: 18500,
      unit: "USD",
      date: "2025-02-15",
      status: "completed",
      description: "Software license",
    },
    {
      id: 8,
      recipient: "Contractor H",
      amount: 7300,
      unit: "USD",
      date: "2025-02-10",
      status: "completed",
      description: "Maintenance work",
    },
  ],
};

export default function PayoutsPage() {
  return <PayoutsContent payoutsData={payoutsData} />;
}
