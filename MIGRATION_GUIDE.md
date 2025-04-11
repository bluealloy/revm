
# v68 tag (revm v22.0.0) -> v69 tag ( revm v23.0.0)


* Removal of `EvmData`.
    * It got flattened and ctx/inspector fields moved directly to Evm, additional layering didn't have purpose.

# v67 tag (revm v21.0.0) -> v68 tag ( revm v22.0.0)


* No code breaking changes
* alloy-primitives bumped to v1.0.0 and we had a major bump because of it.