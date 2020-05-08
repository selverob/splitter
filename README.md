Splitter
============

Splitter is an interactive transaction creator for
[ledger](https://www.ledger-cli.org) CLI accounting utility. It lets you enter
your transactions while enjoying some slight quality-of-life improvements, like
context-aware autocomplete and automatic transaction splitting.

What's automated transaction splitting? Me and my roommate share costs for a lot
of stuff we buy and we want to know how much we owe each other. I use ledger to
store that information, so if we get food for 10€ and share all of it, I create
a transaction looking like:

```
2020-04-06 Lidl
    Expenses:Food     €5.00
    Debts:Roomie      €5.00
    Accounts:Checking
```

In this case, it's pretty simple, but over multiple kinds of items (think Food,
Hygiene, Beer, etc.) it gets cumbersome to keep the rolling totals or use
ledger's arithmetic expression support. So I built splitter, which allows me to
simply add this kind of split transactions.

Usage
---------
Start splitter with a single argument, path to your Ledger file (automatic
loading based on ledger config is not yet implemented). Then you can enter
transactions. Each transaction begins with a standard header, similar to the
Ledger one:

```
2020-03-02 Transaction description
````

Then you can enter commands. There are three commands:
* `a <Account Name> <Currency> <Amount>` - Adds or subtracts the amount from the
  given account
* `s <Account Name> <Account Name> <Currency> <Amount>` - Splits the amount in
  half and adds or subtracts the halves from the given accounts
* `f <Account Name>` - Finalizes (balances) the transaction, adding or
  subtracting the remaining amount from the given account

Transaction entry can be finalized by entering an empty line. The transaction is
then saved into the file. The CLI supports currency and account name
autocompletion, triggered by Tab.

WARNING: Transaction saving is not yet tested completely. I recommend versioning
your Ledger in Git or backing it up, since it's possible it will get wrecked by
the transaction positioning logic.
