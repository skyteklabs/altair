# Copied Instances are spotted by an installation ID in Check-ins

Builds on ADR-0010. Every copy of an Instance shares its Licence and so its Instance ID, which alone can't tell one running copy from a restart. Each running copy therefore generates a random installation ID on first start, stores it with its data, and sends it in every Check-in; one Instance ID reporting two installation IDs at once is a Copied Instance. The installation ID is an implementation detail, not a domain term.

We accept two gaps: a copy goes unseen unless both it and the original Check in, and copies made together with their data carry the same installation ID. We rejected hardware fingerprints for the same reason as ADR-0010: they break whenever containers are rescheduled.
