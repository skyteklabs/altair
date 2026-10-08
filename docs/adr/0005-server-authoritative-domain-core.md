# The domain rules run on the server; the apps are clients

All domain rules (Transfers stay in one Space, Average cost, Budgets, and so on) are implemented once, in a standalone Rust domain crate used by the API. The mobile apps and dashboard send commands and display query results; they never re-implement the rules. Version one requires a connection to record anything. The planned next step is an offline queue: the apps hold Transactions recorded offline and the server re-validates them on sync.

## Considered Options

- **Compiling the Rust core into the apps (UniFFI) for full offline use** was deferred, not rejected: offline editing of shared Spaces turns sync into conflict resolution, which is too much for version one. Keeping the domain core a standalone crate with no infrastructure inside keeps this option open.
