# Instances degrade to read-only and never lock out Active Users

Builds on ADR-0009 and ADR-0010. When an Instance falls outside its Licence, it never refuses to start or hides data; it only stops new writes or new sign-ins, so a Licensee is never locked out of its records and its members' day isn't interrupted.

- **No valid Licence**: a Licence that is missing, malformed, signed by an unknown key, has a bad signature, names another Instance ID or uses an unsupported format version makes the Instance an Expired Instance, the same as a revoked Licence or one past its Grace period. We rejected refusing to start, because a botched Licence file or a key-rotation mistake would then make a bank's records unreadable until Palmy responds. Tampering with the binary is a separate check: a container image that fails its signature still refuses to start (ADR-0010).
- **Active User cap**: at the cap, Users who are already Active Users keep signing in; only new or returning Users are refused. We rejected refusing everyone at the cap, because reaching a commercial limit should never cut off people already using Palmy that month.
