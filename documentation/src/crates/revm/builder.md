
# Evm Builder

Is a helper function that allows easier setting of database external and logic structures.

There is a dependency between Database, External and Spec types so setting Database will reset External and Handle field while setting External field would reset Handler. Note that Database will never be reset.