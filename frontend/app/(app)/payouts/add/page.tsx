import { AddPayoutForm } from "@/components/add-payout-form";

export default function AddPayoutPage() {
  return (
    <div className="flex-1 gradient-bg">
      <div className="container grid flex-1 items-start gap-4 px-4 py-4 md:px-6">
        <div className="mx-auto w-full max-w-[1200px] space-y-6">
          <AddPayoutForm />
        </div>
      </div>
    </div>
  );
}
