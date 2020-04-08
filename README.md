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
