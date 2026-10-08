# Tech stack: Rust API, native apps, React dashboard, REST + OpenAPI

- **API**: Rust with axum, PostgreSQL accessed through sqlx (compile-time-checked SQL, no ORM). Money and Asset units are stored as `NUMERIC`, never floating point.
- **Contract**: REST, with the OpenAPI spec generated from the Rust code (utoipa). Swift, Kotlin and TypeScript clients are generated from that spec, never hand-written.
- **Mobile**: two native apps, Swift + SwiftUI for iOS and Kotlin + Jetpack Compose for Android. No cross-platform framework and no Kotlin Multiplatform, because shared logic lives on the server.
- **Dashboard**: TypeScript + React single-page app using the generated TypeScript client.

## Considered Options

- gRPC/Connect was rejected: protobuf tooling on the web dashboard and in the mobile apps adds friction that REST + generated clients avoids at this size.
- Flutter / React Native and Kotlin Multiplatform were rejected in favour of fully native UI; the domain rules don't need to be shared on-device yet (see ADR-0005).
