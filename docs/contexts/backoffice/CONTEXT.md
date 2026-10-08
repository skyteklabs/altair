# Backoffice

The Backoffice is the business side of Palmy: who can sign in, what they pay for, and how Palmy staff support them through the internal dashboard. Staff never see anything inside a Space (ADR-0006); the finance vocabulary lives in the [Ledger](../ledger/CONTEXT.md) context. Conversations with Users happen in an external helpdesk, outside Palmy.

## Language

### Users

**User**:
A person who signs in to Palmy: on Palmy Cloud through Apple, Google or an emailed one-time code, or on an Instance through its Licensee's identity provider. In the Ledger, a User acts as a Member of Spaces.
_Avoid_: Account, customer, member (outside a Space), profile

**Suspension**:
A block placed by Staff that stops a User from signing in, without deleting anything.
_Avoid_: Ban, lock, deactivation

### Paying

**Plan**:
A level of Palmy features a User can have: Free or Premium.
_Avoid_: Tier, licence (a different concept), package, product

**Free**:
The Plan every User starts on: a Personal space with up to the Wallet limit (5 by default, set by Growth) in a single currency. A Free User can join a Premium Space but cannot create a shared Space.
_Avoid_: Basic, trial

**Premium**:
The paid Plan, which adds shared Spaces, Investing, multiple currencies and unlimited Wallets.
_Avoid_: Pro, plus, paid

**Subscription**:
A User's right to Premium for a period, bought through the App Store or Play Store or granted by a Promo code. It belongs to the User, never to a Space.
_Avoid_: Licence (a different concept), membership, purchase

**Promo code**:
A code, created by Staff, that grants a free period of Premium when a User redeems it; no payment is involved.
_Avoid_: Coupon, voucher, discount code, promo

**Redemption**:
One User's use of a Promo code, which starts a Subscription.
_Avoid_: Claim, use, activation

_Example_: Alice redeems the Promo code LEBARAN26 for 3 months of Premium. Her Subscription makes the household Space she shares with Bob a Premium Space, even though Bob is on Free.

### Staff

**Staff**:
A Palmy employee who signs in to the dashboard with a company Google Workspace identity; Staff are never Users and can never sign in as one.
_Avoid_: Admin (as a person), agent, operator, employee

**Staff role**:
What a Staff member may do: Admin (everything, including managing Staff and revoking Licences), Support (looking up Users, their Subscriptions, Suspensions and deletion on request), Growth (Plans, the Wallet limit and Promo codes) or Sales (issuing and renewing Licences).
_Avoid_: Permission, group, level

**Audit entry**:
A permanent record of one Staff action: which Staff member did what, when, and to what, such as a User, a Promo code, a Licensee or a Licence. Staff cannot edit or delete Audit entries.
_Avoid_: Log, history, activity

### Self-hosting

**Palmy Cloud**:
The Palmy service run by Palmy itself, which Users reach through the store apps and pay for with Subscriptions.
_Avoid_: SaaS, hosted version, main server

**Instance**:
Palmy as run by a Licensee on its own servers at its own Instance address, with its own Users, who sign in through the Licensee's identity provider, and data that Palmy can never read. Every User on an Instance gets all Licensed features; Plans, Subscriptions and Promo codes do not exist there.
_Avoid_: Self-hosted server, deployment, installation, tenant

**Licensee**:
An organisation, such as a bank, cooperative or company, that holds a Licence to run an Instance.
_Avoid_: Customer, client, partner, tenant

**Instance administrator**:
A User of an Instance whom the Licensee's identity provider marks as an administrator, acting for the Licensee inside the Instance; they receive its expiry and Active User cap warnings.
_Avoid_: Admin (a Staff role), operator, Licensee contact

**Licence**:
A document signed by Palmy that permits one Licensee to run one Instance, at one Instance address, until an expiry date, up to its Active User cap and with a set of Licensed features; it can be verified without contacting Palmy.
_Avoid_: License key, subscription, activation, serial

**Active User**:
A User of an Instance who has signed in within the last 30 days; this is what the Active User cap counts.
_Avoid_: Seat, licensed user, MAU

**Active User cap**:
The most Active Users an Instance may have.
_Avoid_: Seat limit, user limit, licence cap

**Licensed feature**:
A part of Palmy, such as Investing or shared Spaces, that a Licence switches on for its Instance.
_Avoid_: Module, add-on, entitlement

**Instance address**:
The domain of an Instance, set by its Licensee, which a User enters or scans in the official app to use that Instance instead of Palmy Cloud.
_Avoid_: Server URL, endpoint, workspace

**Check-in**:
An Instance's optional, periodic report to Palmy for Licence renewal, revocation, usage counts and spotting Copied Instances; it never carries Ledger data.
_Avoid_: Heartbeat, ping, phone-home, telemetry

**Copied Instance**:
An Instance running in more than one place at once.
_Avoid_: Clone, pirated instance, duplicate installation

**Grace period**:
The 14 days after a Licence expires during which its Instance works normally but shows a renewal warning.
_Avoid_: Buffer, overdue period

**Expired Instance**:
An Instance with no valid Licence, as ADR-0011 defines it, including one revoked or expired past its Grace period: all data stays readable, but nothing new can be recorded until a valid Licence is installed.
_Avoid_: Locked instance, disabled instance, unlicensed

### Not yet modelled

White-label apps per Licensee, sign-in built into an Instance (for Licensees without an identity provider), and selling Licensed features separately are deliberately left for later.
