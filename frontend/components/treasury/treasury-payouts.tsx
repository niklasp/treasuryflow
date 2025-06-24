import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  CardDescription,
} from "../ui/card";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "../ui/table";
import { Button } from "../ui/button";

interface Payout {
  id: number;
  recipient: string;
  amount: number;
  unit: string;
  date: string;
}

export function TreasuryPayouts({ treasuryData }: { treasuryData: any }) {
  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between">
        <div>
          <CardTitle>Recent Payouts</CardTitle>
          <CardDescription>A list of your recent payouts</CardDescription>
        </div>
        <Button variant="outline" size="sm">
          View All
        </Button>
      </CardHeader>
      <CardContent>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Recipient</TableHead>
              <TableHead>Amount</TableHead>
              <TableHead>Date</TableHead>
              <TableHead className="text-right">Status</TableHead>
            </TableRow>
          </TableHeader>
          {!treasuryData.payouts || treasuryData.payouts.length === 0 ? (
            <TableBody>
              <TableRow>
                <TableCell colSpan={4} className="text-center">
                  No payouts found
                </TableCell>
              </TableRow>
            </TableBody>
          ) : (
            <TableBody>
              {treasuryData.payouts.map((payout: Payout) => (
                <TableRow key={payout.id}>
                  <TableCell className="font-medium">
                    {payout.recipient}
                  </TableCell>
                  <TableCell>
                    {payout.unit} {payout.amount.toLocaleString()}
                  </TableCell>
                  <TableCell>
                    {new Date(payout.date).toLocaleDateString()}
                  </TableCell>
                  <TableCell className="text-right">
                    <span className="inline-flex items-center rounded-full bg-primary/10 border border-primary/20 px-2.5 py-0.5 text-xs font-medium text-primary">
                      Completed
                    </span>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          )}
        </Table>
      </CardContent>
    </Card>
  );
}
