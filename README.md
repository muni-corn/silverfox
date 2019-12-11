# mvelopes

mvelopes is a command-line double-entry plain-text
rootin-tootin accounting tool like ledger-cli or hledger.
But unlike other plain-text accounting tools, mvelopes is
designed around "envelope budgeting". Its main concern is
helping you budget without overspending as well as keeping a
good record of your finances.

More will be here once the project matures and is
functional. Currently, it doesn't do anything.

## What sets mvelopes apart from other tools?

### Expenses, savings goals, and more

mvelopes works by moving money from your available balances
into "envelopes" every day. Little by little (or lots by
lots), it puts money towards anything you might be spending
money on: rent, electricity, or a new boat (whatever; you
name it).

For any recurring expenses, mvelopes keeps two envelopes:
one for what's ready to be spent, and another for the next
time an expense is due. This ensures a separation of
concerns from due date to due date.

#### Methods of saving money

mvelopes lets you choose from two methods of automatically
moving money into envelopes: aggressive or conservative.
Aggressive saving moves as much money as possible into
envelopes as soon as possible. Conservative saving moves
money in little by little every day, saving up just in time
for a due date.

Of course, you can also move money manually and disable
automated saving on a per-envelope basis.

### Required transaction statuses

Each transaction entry must be entered with one of three
statuses:

```
?       Pending
!       Cleared
*       Reconciled
```

mvelopes will let you know if a transaction is not marked
with one of these statuses.

### Required account definitions

mvelopes requires you to define a list of accounts. You can
use the `account` property to do this:

```
account assets:checking
account assets:savings
account expenses:groceries
account expenses:auto:gas
account expenses:home:rent
account expenses:home:electricity
account income:salary
account income:gifts
```

## Journal format

If you've used ledger-cli or hledger, mvelopes uses a
similar formatting syntax for journals:

```
2019/08/02 * Groceries
    assets:checking     -30
    expenses:groceries   30
```

More specifically:

```
<date> <status> <description> [<payee>]
    <account> <amount>
    <account> <amount>
```

> Note: While mvelopes has formatting extremely similar to
> hledger or ledger-cli, an mvelopes file is not necessarily
> compatible with an hledger or ledger-cli file. You'll
> notice that (for example) an equal sign (=) in a posting
> (an account-amount line in a transaction entry) has two
> very different meanings between hledger and mvelopes:
> hledger uses it for balance assertions, and mvelopes uses
> it for cost assertions.

Only at least one space is required after `<account>`.
`<payee>` is not required, but if provided in square
brackets as above, can be queried in mvelopes's output.

#### Comments

Comments in mvelopes are done with either semicolons (`;`)
or a double-slash (`//`), which is preferred and will be
used when using `mvelopes format`:

```
2019/08/02 ! Restaurant [Fancy's]
    assets:checking     -140        // Might've ordered too much
    expenses:dining      140
```

### Currencies and prices

If a currency symbol isn't included, mvelopes considers it
your default currency. You can include currency symbols in
your transaction:

```
2019/08/02 * Bought crypto
    assets:checking     $-100
    assets:crypto:btc       0.012345 BTC
```

Note that in the above transaction, mvelopes will
automatically balance and infer that the total cost of
0.012345 BTC (Bitcoin) was \$100. You can define the total
cost of the BTC manually as well:

```
2019/08/02 * Bought crypto
    assets:checking     $-100
    assets:crypto:btc       0.012345 BTC = $100
```

Or, you can define the price per unit:

```
2019/08/02 * Bought crypto
    assets:checking     $-100
    assets:crypto:btc       0.012345 BTC @ $8100.45
```

Information about using envelopes on transactions and moving
money to them manually is outlined below.

## Envelopes

### Configuration

Here's an example of how to configure envelopes with
mvelopes. Envelopes are created under asset accounts:

```
account assets:checking
    expense rent due every 15th                     // due the 15th of every month
        amount 1000                                 // for $1000
        funding aggressive                          // use aggressive funding

    expense electricity due every other 15th        // due the 15th of every other month
        amount 100                                  // for $100
        for expenses:home:electricity               // automatically moves money when expenses:home:electricity is used
        funding conservative                        // use conservative funding
        
    expense food due every 1st                      // due the 1st of every month
        amount 300                                  // for $300
        for expenses:groceries                      // automatically moves money when expenses:groceries is used
        for expenses:dining                         // and expenses:dining
```

> Note: the `funding` option is optional. If omitted, mvelopes
> won't move money automatically. 

We can also do envelopes for goals:

```
account assets:savings
    goal camera by 2020/12/25                       // "due" and "by" are interchangeable
        amount 1000                                 // you get the gist!
        funding conservative

    goal "new boat" by 2025/06/28
        amount 35000
        funding conservative
```

Really, the `expense` and `goal` keywords are both
interchangeable. They do the same thing. And, as mentioned,
`due`, `by`, and even `due by` are the same. But not `by
due`. That makes no sense. mvelopes will throw an error.

A couple of other points to note:

- Expenses and goals can co-exist under the same account
- If your expense or goal name includes whitespace, it must
  be wrapped in quotes.

### Manual envelope movements

```
2019/08/02 * Groceries
    assets:checking         -50
    expenses:groceries       50
    envelope food
```

mvelopes infers that this transaction should take money from
the `food` envelope under the `assets:checking` account.

Of course, things can get a little more complicated:

```
2019/08/02 * Groceries with cash back
    assets:checking               -70
    assets:cash                    20
    expenses:groceries             50
    envelope assets:checking food -50
```

Since this transaction includes two postings from assets,
mvelopes can't infer which account from which to use an
envelope. 

## Fun facts

(this was originally going to work alongside hledger but then
I decided to make a full-blown accounting program because I
decided I didn't like Haskell and I wanted to learn Rust)

So, this project features my first attempts at Rust. I'm
learning as I go. Good experience so far :)

## Donate

Asking for money isn't my thing, but if you're interested in
helping to fund my education, rent, food, or even just a hot
chocolate, you're welcome to send me Bitcoin or Ethereum:

BTC

```
35yDrjaUwdFgLfJjboYxcY1mNLm1EvMLys
```

ETH

```
0x05d639861B8B7058ae237B41ef71ca2291A295e3
```

Thank you! Your interest in this project means the world for
me.
