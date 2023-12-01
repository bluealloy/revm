
# Evm Builder

Is a helper that allows easier setting of database, external and logic structures.

It ties dependency between Database, External and Spec and allows setting custom logic. As there is a dependency between them setting Database will reset External and Handle field while setting External field would reset Handler. Note that Database will never be reset.

Simple example of using `EvmBuilder` is

```

Evm::build().with_empty_db().with_empty_external()
```