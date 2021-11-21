# EVM console debugger

This can be very interesting, it gives you ability to step, modify stack/memory, set breakpoints and do everything what you would expect from standard debugger, with addition of rewinding step and contract calls. You can connect to exteranl web3 supported API and fetch for example live state from mainnet via infura, or you can set data local manupulation, either way this should be useful.

This binary will be console based and interaction will be done via console inputs, this is great showcase this is first step.

commands:
`step`
`continue`
`breakpoint <contract> <pc>`
`rewind call`
`rewind opcode`
`stack`
`stack set <index> <value>`
`memory`
`memory set <offset> <value>`
`state`
`state <index> <value>`
`account <address>`
`account <address> balance <new_balance>`
`account <address> nonce <new_nonce>`
`storage <index>`
`storage set <index> <value>`
what to do with cold/hot access, do we add clear hot command?