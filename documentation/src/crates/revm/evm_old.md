# EVM Module Documentation

This document provides the documentation for the `EVM` module.

## The `EVM`

The primary struct in this module is `EVM`. The EVM struct is generic over a type DB. This means that when you use the EVM struct, you can specify the type that DB should represent. It adds flexibility to the struct, allowing it to store different types of databases or data structures in the db field depending on the use case. The `EVM` struct enables `transact` to update the state directly to the database. Additionally, it allows the user to set all environment parameters.

The parameters that can be set are divided between `Config`, `Block`, and `Transaction` (tx). For transacting on the EVM, you can call `transact_commit` that will automatically apply changes to the database.
