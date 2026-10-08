# Ledger

The Ledger is the heart of Palmy, a finance tracker: Users record where their money is and how it moves, alone or together with others. Palmy never holds or moves real money; it only keeps a record of money held elsewhere. "Altair" is the internal codename for the repo, not a domain term. Logins, Plans and Staff belong to the [Backoffice](../backoffice/CONTEXT.md) context.

## Language

**Palmy**:
The product as users see it. Only the product name; not used for any concept inside it.
_Avoid_: Altair (codename only), app

### Who owns what

**Premium Space**:
A Space in which at least one Member has an active Premium Subscription (see Backoffice), unlocking sharing, Investing, multiple currencies and unlimited Wallets.
_Avoid_: Paid space, pro space

**Lapsed Space**:
A shared Space, or one with Premium-only Wallets, in which no Member has Premium any more. Nothing in it is deleted or hidden, but Premium-only actions (new Transactions in a shared Space, Buys and Sells, Wallets beyond the Free limit) are blocked until a Member has Premium again.
_Avoid_: Expired space, locked space, frozen space

**Space**:
A named group of Members that owns Wallets, Categories, Budgets and Transactions; every Member sees everything in it, and it is a closed book with its own Home currency.
_Avoid_: Household, group, workspace, team

**Personal space**:
The Space every User has to themselves, which is never shared.
_Avoid_: Private wallet, my account

**Member**:
A User who belongs to a Space; all Members can see and edit everything in it.
_Avoid_: User (when talking about a role within a Space), participant, account

**Owner**:
The one Member of a Space who manages its membership and can delete it; the Owner can only leave by handing ownership to another Member or deleting the Space.
_Avoid_: Admin, creator

**Former Member**:
A User who has left a Space; Transactions they were Made by or Recorded by stay in the Space and still name them.
_Avoid_: Deleted user, ex-member

### Where money is

**Wallet**:
A labelled place where money is kept outside Palmy, such as a bank account, a cash pocket or an e-wallet, tracked in a single currency and owned by exactly one Space.
_Avoid_: Account, pocket, source

**Liability wallet**:
A Wallet whose balance is money owed, such as a credit card or a loan.
_Avoid_: Debt, credit account, negative wallet

**Net worth**:
The total of a Space's non-liability Wallets minus its Liability wallets, in the Home currency at current exchange rates.
_Avoid_: Balance, total

**Deposit wallet**:
A Wallet holding a time deposit: a principal locked until a maturity date, whose interest is recorded as Income.
_Avoid_: Time deposit account, savings, investment

**Home currency**:
The single currency a Space reports in; amounts from Wallets in other currencies are converted into it.
_Avoid_: Base currency, default currency, main currency

### How money moves

**Transaction**:
A single recorded movement or change of money within one Space; it is exactly one of Income, Expense, Transfer, Adjustment, Revaluation, Buy or Sell.
_Avoid_: Entry, record, payment

**Income**:
A Transaction where money, or units of an Asset (such as a staking reward or airdrop), enters a Wallet from outside the Space; units are valued at the Price on that date, which also becomes their Average cost.
_Avoid_: Deposit, earning, credit

**Expense**:
A Transaction where money leaves the Space for something it paid for. Spending on a Liability wallet is an Expense from that Wallet.
_Avoid_: Spending, payment, debit, cost

**Out-of-pocket expense**:
An Expense of a Space that a Member paid with money from outside the Space, so it touches none of the Space's Wallets but still counts against its Budgets.
_Avoid_: Reimbursable, fronted expense, external payment

**Refund**:
An Expense going the other way: money coming back for something the Space paid for, recorded in the original Expense's Category so it reduces that Category's spending.
_Avoid_: Return, reimbursement, cashback, negative expense

**Transfer**:
A Transaction that moves money, or units of an Asset, between two Wallets of the same Space; it is neither Income nor Expense, and across currencies it has a sent amount and a received amount. Paying off a Liability wallet is a Transfer into it.
_Avoid_: Move, top-up

**Adjustment**:
A Transaction that corrects a Wallet's balance to match reality after a recording mistake or drift, without counting as Income or Expense.
_Avoid_: Correction, reconciliation, fix

**Revaluation**:
A Transaction recording a market change in the value of an Investment wallet without Holdings; it changes Net worth but is neither Income nor an Adjustment.
_Avoid_: Gain, loss, return, adjustment

**Made by**:
The Member who actually paid or received the money in a Transaction, which may differ from who recorded it.
_Avoid_: Owner, payer, spender

**Recorded by**:
The Member who entered a Transaction into Palmy.
_Avoid_: Author, creator

_Example_: Alice moves money from her Personal space into the household Space. That is two Transactions: an Expense in her Personal space and an Income in the household Space. It is never a Transfer, because a Transfer stays inside one Space.

_Example_: Alice buys the household's groceries on her personal credit card. The household Space records an Out-of-pocket expense Made by Alice; her Personal space records the Expense on her card.

### Investing

**Investment wallet**:
A Wallet whose value moves with the market. Either it is made up of Idle funds plus Holdings, such as an Indonesian stock brokerage, a US broker, a crypto exchange or a cold wallet, and its value is its Idle funds plus the current value of its Holdings; or it has no Holdings and its value is kept as a single amount by Revaluations, such as a pension fund. Never both.
_Avoid_: Portfolio, savings wallet, asset account, brokerage account, RDN

**Idle funds**:
The uninvested cash in an Investment wallet, in the Wallet's currency; never negative. A cold wallet or a drawer of physical gold has Holdings but no Idle funds.
_Avoid_: Cash balance, buying power, RDN balance

**Asset**:
Something that can be held in units and has a price in its own currency, such as a stock, an ETF, a mutual fund, a bond, a crypto coin or gold; Palmy covers Indonesian and US markets.
_Avoid_: Security, instrument, ticker, stock

**Holding**:
The units of one Asset held in one Investment wallet, together with its Average cost; units are never negative.
_Avoid_: Position, investment, stock

**Price**:
The current value of one unit of an Asset, either looked up from a price feed or entered by a Member when no feed covers the Asset.
_Avoid_: Quote, rate, NAV (except when talking about mutual funds specifically)

**Buy**:
A Transaction in an Investment wallet that turns Idle funds into units of a Holding, dated on the trade date regardless of when it settles; it is neither Income nor Expense.
_Avoid_: Purchase, order, invest

**Sell**:
A Transaction in an Investment wallet that turns units of a Holding back into Idle funds and locks in a Realised gain, dated on the trade date; it is neither Income nor Expense.
_Avoid_: Sale, redeem, liquidate

**Average cost**:
The price paid per unit of a Holding, including Trade costs, averaged over all its Buys; a Sell does not change it, and a Transfer of units carries it along.
_Avoid_: Cost basis, average price, purchase price

**Unrealised gain**:
How much a Holding's current value exceeds its units × Average cost; negative when it is a loss.
_Avoid_: Paper profit, floating profit/loss, return

**Realised gain**:
How much a Sell's proceeds, after its Trade costs, exceed the units sold × Average cost; negative when it is a loss. It is not Income, because Net worth already rose with the Price.
_Avoid_: Profit, capital gain, return

**Trade costs**:
The broker fee and tax recorded on a Buy or Sell. Kept as separate amounts and reported on their own, they raise the Average cost on a Buy and cut the proceeds on a Sell, and are not Expenses, so they never count against Budgets.
_Avoid_: Commission, charges, investment expense

**Stock split**:
An event on a Holding that multiplies its units and divides its Average cost by the same factor, without any money moving; a reverse split is a Stock split with a factor below one.
_Avoid_: Split (reserved for splitting a Transaction across Categories), corporate action

**Dividend**:
Income into an Investment wallet's Idle funds paid by an Asset, recorded after any tax withheld, in a Dividend Category; bond coupons are recorded the same way.
_Avoid_: Payout, distribution, coupon (as a separate concept)

_Example_: Alice holds 100 units of BBCA at an Average cost of 9,000. Ignoring Trade costs, she sells 40 at 10,000: Idle funds rise by 400,000, the Realised gain is 40,000, and the remaining 60 units keep their Average cost of 9,000.

### What money is for

**Category**:
A label owned by a Space that says what an Income or Expense was for, such as Food or Salary. Each Category is either an Income Category or an Expense Category and may have one level of sub-Categories. Every Income and Expense has exactly one Category; Transfers, Adjustments, Revaluations, Buys and Sells never have one.
_Avoid_: Tag, type, group

**Month**:
A Space's monthly period, running from the start day the Space chooses (e.g. the 25th) to the day before it in the next calendar month.
_Avoid_: Period, cycle, pay period

**Budget**:
A spending limit for one Expense Category in a Space for each Month; a Budget on a Category includes spending in its sub-Categories, which may have Budgets of their own.
_Avoid_: Limit, plan, allowance

**Underspend**:
The part of a Budget left unspent at the end of a Month. It is reported per Month and Category and never builds up into a running total.
_Avoid_: Savings, leftover, surplus

_Example_: Food's Budget is 2,000,000 and March's spending is 1,500,000, so the Underspend is 500,000. Palmy can offer to record a Transfer of it into an Investment wallet, which the user confirms with a real source Wallet.

### Not yet modelled

Debts between Members (who owes whom, settling up), money owed to a Space that a Member received personally, recurring Transactions, saving goals, splitting one Transaction across Categories, read-only Members, corporate actions other than Stock splits (rights issues, bonus shares), per-Asset total return including Dividends, investment tax reporting, pending orders and settlement, margin and short selling, and importing or syncing Transactions are deliberately out of the core for now.
