# Licences are a signed envelope, and Invalid checks run in a fixed order

Builds on ADR-0009 and ADR-0010. A Licence file is `{payload, signature}`: `payload` is base64url of the exact JSON bytes that Ed25519 signed, and `signature` is the raw 64-byte signature, base64url. The Instance verifies those bytes as-is and never re-encodes them, so canonicalisation bugs can't reach verification. The key ID, format version, Licensee ID and name, Instance ID, Instance address, dates, Active User cap and Licensed features all sit inside the signed payload, so changing any of them breaks the signature. Licensed features are an open set of names: an Instance acts on the names it knows and ignores the rest, so a newer Licence doesn't lock an older Instance out. Unknown fields in the payload are still rejected, and the Active User cap must be at least 1.

The Instance checks in a fixed order: format version, then strict parse, then key ID, then signature, then Instance ID, then the Licence term. The format version and key ID are read from the payload before the signature is checked, using a loose first pass that grants nothing; they become trusted only once the signature verifies. Each `Invalid` reason therefore has exactly one trigger, and a Licence signed for another Instance by a trusted key reports `wrong Instance ID`.

## Considered Options

- **Canonical JSON (RFC 8785), signed after re-encoding** was rejected: the Instance would re-encode before verifying, which is where canonicalisation bugs come from.
- **A closed enum of Licensed features**, bumping the format version for each new feature, was rejected: a new feature would make an older Instance read the Licence as `malformed` and go read-only over an upgrade.
- **Signature before the strict parse** was rejected: garbage bytes would report `bad signature` instead of `malformed`.
