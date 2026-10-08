# Palmy tracks money; it never holds or moves it

Despite being pitched as a "wallet", Palmy is a tracker only: every Wallet represents money held somewhere else (a bank, cash, an e-wallet), and Palmy only records what the user says happened. We chose this over being a custodial e-wallet because holding and moving money brings licensing, KYC and a payments ledger that a personal-finance tracker doesn't need.

## Consequences

- Palmy never creates a Transaction the user hasn't confirmed. Suggestions (e.g. moving an Underspend into an Investment wallet) are offers the user accepts, not automatic movements, so Wallets keep matching reality.
- Becoming a real wallet later would be a new product decision with its own ADR, not an extension of the current Wallet concept.
