import Link from "next/link";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { AlertTriangle, ArrowLeft } from "lucide-react";

export default function TreasuryNotFound() {
  return (
    <div className="flex-1 gradient-bg">
      <div className="container grid flex-1 items-center justify-center px-4 py-12 md:px-6">
        <div className="mx-auto max-w-md">
          <Card className="border-white/5 bg-black/40 backdrop-blur-md">
            <CardHeader className="text-center">
              <div className="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-yellow-500/10">
                <AlertTriangle className="h-6 w-6 text-yellow-500" />
              </div>
              <CardTitle className="text-xl">Treasury Not Found</CardTitle>
            </CardHeader>
            <CardContent className="text-center space-y-4">
              <p className="text-muted-foreground">
                The treasury you're looking for doesn't exist or may have been
                removed.
              </p>
              <div className="flex flex-col gap-2 sm:flex-row sm:justify-center">
                <Link href="/dashboard">
                  <Button
                    variant="outline"
                    className="w-full sm:w-auto border-white/5 bg-black/20 hover:bg-black/40"
                  >
                    <ArrowLeft className="mr-2 h-4 w-4" />
                    Back to Dashboard
                  </Button>
                </Link>
                <Link href="/create-treasury">
                  <Button className="w-full sm:w-auto primary-gradient hover:primary-gradient-hover glow">
                    Create Treasury
                  </Button>
                </Link>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  );
}
