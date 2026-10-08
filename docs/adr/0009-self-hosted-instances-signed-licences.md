# Organisations can self-host Palmy under an offline-verifiable signed Licence

Besides Palmy Cloud (the service we run, ADR-0007), organisations such as banks, cooperatives and companies can run their own Instance of Palmy on their own servers under a Licence. A Licence is a file signed with Palmy's private Ed25519 key, naming the Licensee, the Instance, the expiry date and the limits; the Instance verifies it locally against public keys embedded in the binary, so it works fully offline or air-gapped. Instances whose Licensee allows it also check in with the licence server for renewal, revocation and usage reporting. When a Licence expires, the Instance becomes read-only: data stays readable, new writes are blocked. An organisation is never locked out of its records.

## What we protect against, and what we don't

The threats are piracy (running without a valid Licence or beyond its limits) and tampering (modifying the binary to bypass limits or alter calculations). Defences: Licences can't be forged without the private key; Licence checks happen in several places; release builds are stripped and optimised; sensitive constants are encrypted in the binary; container images are signed and the Instance refuses to start if it has been tampered with; Licence terms allow audits.

We do **not** claim the binary cannot be reverse engineered: anything that runs on a customer's machine can eventually be analysed and patched. The cryptographic signature and the contract are the real protection; obfuscation only raises the cost. Commercial control-flow obfuscators were rejected for now because of cost, runtime overhead and unreadable crash reports.

## Data

Encryption means TLS in transit and encryption at rest under keys the Licensee holds. Nobody at Palmy can read an Instance's data, and check-ins never carry Ledger data (consistent with ADR-0006). End-to-end encryption on the phones was rejected: it would force the domain rules onto the devices, which ADR-0005 deferred.

## Consequences

ADR-0007 (Firebase Auth, GCP Jakarta) and ADR-0008 (store-only billing) apply to Palmy Cloud only, not to Instances.
