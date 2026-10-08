# One monorepo for API, dashboard and mobile apps

The Rust API, the internal dashboard, the iOS app and the Android app all live in this one repo (`api/`, `dashboard/`, `ios/`, `android/`), sharing one glossary, one ADR log, one Linear project and one API contract. We chose this over separate repos so that an API change and the client updates that depend on it land in a single PR, and so agents see the whole system. The cost is that CI must build only what changed.
