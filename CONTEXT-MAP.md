# Context Map

## Contexts

- [Ledger](./docs/contexts/ledger/CONTEXT.md): Spaces, Wallets, Transactions, Budgets and Investing — what Users record about their money
- [Backoffice](./docs/contexts/backoffice/CONTEXT.md): Users' logins, Plans, Subscriptions, Promo codes, Staff, and Licences for self-hosted Instances — the business side, run from the internal dashboard

## Relationships

- **Backoffice → Ledger**: the Ledger knows Users only by ID. Whether a Space is a Premium Space comes from Backoffice: a Space is Premium while at least one of its Members has an active Subscription.
- **Ledger ↛ Backoffice**: Backoffice and Staff never read Ledger data (ADR-0006); only aggregate, anonymised metrics may cross.
- **User deletion**: deleting a User in Backoffice deletes their Personal space in the Ledger; shared Spaces keep their Transactions and name them as a Former Member.
- **Instances**: an Instance runs the Ledger on a Licensee's servers. The only thing that crosses back to Palmy's Backoffice is an optional Check-in, never Ledger data (ADR-0009).
