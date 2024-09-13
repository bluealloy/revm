# EVM Memory

Is a memory localized to the current Interpreter context. Interpreter context is a call or create frame. It is used by opcodes to store or format data that are more than 32 bytes long, for example calls to format input, return output or for logs data. Revm has a shared memory between all the Interpreters but Interpreter loop only see the part it is allocated to it.

Extending memory is paid by the gas. It consumes 3 gas per word plus square of the number of words added divided by `512` (`3*N+ N^2/512`). There is no limit on the size of the memory, but it is limited by logarithmic growth of the gas cost. For 30M there is a calculated max memory of 32MB (Blog post by ramco: [Upper bound for transaction memory](https://xn--2-umb.com/22/eth-max-mem/)).

## Opcodes

Here is a list of all opcodes that are reading or writing to the memory. All read on memory can still change the memory size by extending it with zeroes. Call opcodes are specific as they read input before the call but also write their output after the call (if call is okay and there is an output to write) to the memory.

These opcodes read from the memory:
* RETURN
* REVERT
* LOG
* KECCAK256
* CREATE
* CREATE2
* CALL
* CALLCODE
* DELEGATECALL
* STATICCALL

These opcodes change the memory:
* EXTCODECOPY
* MLOAD
* MSTORE
* MSTORE8
* MCOPY
* CODECOPY
* CALLDATACOPY
* RETURNDATACOPY
* CALL
* CALLCODE
* DELEGATECALL
* STATICCALL
