# Copied Instances are spotted by a per-install ID in Check-ins

Builds on ADR-0010. Every copy of an Instance shares its Licence and so its Instance ID, which alone can't tell one install from a restart. Each install therefore generates a random installation ID on first start, stores it with its data, and sends it in every Check-in; one Instance ID reporting two installation IDs at once is a Copied Instance. The installation ID is an implementation detail, not a domain term.

We accept two gaps: copies that never Check in, and copies made together with their data (they carry the same installation ID). We rejected hardware fingerprints for the same reason as ADR-0010: they break whenever containers are rescheduled.
