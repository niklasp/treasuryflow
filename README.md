# Treasury Payouts

A flexible treasury management system supporting vested, recurring, and instant payouts with multi-signature governance and payout categories.

## Project Structure

payouts/
├── contract_treasury_v5/ # Smart contract implementation with inkv5
├── contract_treasury_v6/ # Smart contract implementation with inkv6 + pop cli
├── frontend/ # Next.js 14 frontend application
│ ├── app/ # App Router pages
│ │ ├── (auth)/ # Authentication routes
│ │ ├── (dashboard)/ # Dashboard routes
│ │ └── api/ # API routes
│ ├── components/ # Reusable components
│ │ ├── auth-wizard/ # Authentication components
│ │ ├── dashboard/ # Dashboard components
│ │ └── ui/ # UI components
│ ├── lib/ # Utility functions
│ ├── services/ # API services
│ └── types/ # TypeScript types

## Features

### Smart Contract

- Multi-signature governance
- Support for different payout types:
  - Scheduled payouts
  - Recurring payouts
  - Vested payouts
- Aggregated payouts to reduce fees
- Asset management (native and registered assets)
- Event-based monitoring

### Frontend

- Next 15 application

## Getting Started

### Contracts

#### v5

```bash
cd contract_treasury_v5
cargo contract build
cargo test
```

#### v6

```bash
### uses pop
cd contract_treasury_v6
pop build
pop test
```

### Frontend

1. Start the development server

```bash
cd frontend
pnpm i
pnpm dev
```

2. Deploy the contract

TODO

## Acknowledgments

- [ink!](https://use.ink/) - Smart contract framework
- [Papi](https://papi.how) - Polkadot API
- [Next.js](https://nextjs.org/) - React framework
- [Shadcn UI](https://ui.shadcn.com/) - UI components
