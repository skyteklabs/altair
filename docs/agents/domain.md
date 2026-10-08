# Domain Docs

How the engineering skills should consume this repo's domain documentation when exploring the codebase.

## Before exploring, read these

- **`CONTEXT-MAP.md`** at the repo root: it points at one `CONTEXT.md` per context under `docs/contexts/`. Read each one relevant to the topic.
- **`docs/adr/`**: read ADRs that touch the area you're about to work in.

If any of these files don't exist, **proceed silently**. Don't flag their absence; don't suggest creating them upfront. The `/domain-modeling` skill (reached via `/grill-with-docs` and `/improve-codebase-architecture`) creates them lazily when terms or decisions actually get resolved.

## File structure

This repo is multi-context. Glossaries live under `docs/contexts/` rather than next to the code, because the API, dashboard, iOS and Android code all use both vocabularies:

```
/
├── CONTEXT-MAP.md                     ← lists the contexts and how they relate
├── docs/
│   ├── adr/                           ← all decisions, system-wide and per-context
│   └── contexts/
│       ├── ledger/CONTEXT.md          ← Spaces, Wallets, Transactions, Budgets, Investing
│       └── backoffice/CONTEXT.md      ← Users, Plans, Subscriptions, Promo codes, Staff
├── api/                               ← Rust workspace
├── dashboard/                         ← React internal dashboard
├── ios/
└── android/
```

## Use the glossary's vocabulary

When your output names a domain concept (in an issue title, a refactor proposal, a hypothesis, a test name), use the term as defined in `CONTEXT.md`. Don't drift to synonyms the glossary explicitly avoids.

If the concept you need isn't in the glossary yet, that's a signal: either you're inventing language the project doesn't use (reconsider) or there's a real gap (note it for `/domain-modeling`).

## Flag ADR conflicts

If your output contradicts an existing ADR, surface it explicitly rather than silently overriding:

> _Contradicts ADR-0007 (event-sourced orders), but worth reopening because…_
