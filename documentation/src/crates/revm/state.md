# State implementations

State inheritis the `Database` trait and implements fetching of external state and storage. and various functionality on output of the EVM execution, most notable caching changes while execution multiple transactions.