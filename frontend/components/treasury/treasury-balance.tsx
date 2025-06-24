import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  CardDescription,
} from "../ui/card";
import { ResponsiveContainer } from "recharts";
import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  Tooltip,
  CartesianGrid,
} from "recharts";

export function TreasuryBalance({ treasuryData }: { treasuryData: any }) {
  if (!treasuryData) {
    return <div>No Treasury Data Found</div>;
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>Treasury Balance</CardTitle>
        <CardDescription>6-month treasury balance history</CardDescription>
      </CardHeader>
      <CardContent>
        {/* <div className="h-[300px]">
          <ResponsiveContainer width="100%" height="100%">
            <AreaChart
              data={treasuryData.chartData}
              margin={{
                top: 10,
                right: 30,
                left: 0,
                bottom: 0,
              }}
            >
              <CartesianGrid
                strokeDasharray="3 3"
                stroke="rgba(255,255,255,0.1)"
              />
              <XAxis dataKey="name" stroke="rgba(255,255,255,0.5)" />
              <YAxis stroke="rgba(255,255,255,0.5)" />
              <Tooltip
                contentStyle={{
                  backgroundColor: "rgba(0,0,0,0.8)",
                  borderRadius: "0.75rem",
                  border: "1px solid rgba(255,255,255,0.1)",
                  color: "#fff",
                }}
              />
              <Area
                type="monotone"
                dataKey="amount"
                stroke="#6b46c1"
                fill="url(#colorGradient)"
                fillOpacity={0.6}
              />
              <defs>
                <linearGradient id="colorGradient" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="5%" stopColor="#6b46c1" stopOpacity={0.8} />
                  <stop offset="95%" stopColor="#6b46c1" stopOpacity={0.1} />
                </linearGradient>
              </defs>
            </AreaChart>
          </ResponsiveContainer>
        </div> */}
      </CardContent>
    </Card>
  );
}
