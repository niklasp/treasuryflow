import { CreateTreasuryForm } from "@/components/create-treasury-form";

export default function CreateTreasuryPage() {
  return (
    <div className="flex-1 gradient-bg">
      <div className="container grid flex-1 items-start gap-4 px-4 py-12 md:px-6">
        <div className="mx-auto w-full max-w-[800px] space-y-6">
          <CreateTreasuryForm />
        </div>
      </div>
    </div>
  );
}
