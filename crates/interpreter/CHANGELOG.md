# v1.2.0
date: 28.09.2023

Summary:
* Cancun support:
  * EIP-7516: BLOBBASEFEE opcode
  * EIP-4844: Shard Blob Transactions
  * EIP-1153: Transient storage opcodes
  * EIP-5656: MCOPY - Memory copying instruction
* Rename `SHA3` to `KECCAK256`, this can potentially break some tracers.
* Refactor opcodes and Interpreter dispatch loop. Better performance.
* optimize stack usage for recursive `call` and `create` programs.
    This brings down the native stack usage as calls are in recursion.

Full git log:
* f79d0e1 - feat: Optimism execution changes (#682) (16 hours ago) <clabby>
* d03dfcb - Improve wording and fix typos (#749) (25 hours ago) <Paul Razvan Berg>
* 2c556c0 - refactor: say "warm" instead of "hot" (#754) (25 hours ago) <Paul Razvan Berg>
* 8206193 - feat: add "kzg" as a separate feature (#746) (2 hours ago) <DaniPopes>
* 516f62c - perf(interpreter): remove dynamic dispatch from all instructions (#739) (5 days ago) <DaniPopes>
* 26af13e - EIP-7516: BLOBBASEFEE opcode (#721) (5 days ago) <rakita>
* 36e71fc - fix: dont override instruction result (#736) (6 days ago) <rakita>
* d926728 - perf: refactor interpreter internals and cleanup (#582) (6 days ago) <DaniPopes>
* fa13fea - feat: implement EIP-4844 (#668) (11 days ago) <DaniPopes>
* 190f90e - Never inline the prepare functions (#712) (2 weeks ago) <Valentin Mihov>
* 7eacc3a - chore: implement `Default` for other databases (#691) (3 weeks ago) <DaniPopes>
* 616cc7e - chore(cfg): convert chain_id from u256 to u64 (#693) (3 weeks ago) <Lorenzo Feroleto>
* a95a298 - chore: accept byte slice as input (#700) (3 weeks ago) <Matthias Seitz>
* f6c9c7f - chore: deprecate `RefDBWrapper` (#696) (3 weeks ago) <DaniPopes>
* f2929ad - chore(deps): bump proptest-derive from 0.3.0 to 0.4.0 (#652) (4 weeks ago) <dependabot[bot]>
* 37b0192 - perf(interpreter): improve i256 instructions (#630) (4 weeks ago) <DaniPopes>
* 214e65d - chore(interpreter): improve gas calculations (#632) (5 weeks ago) <DaniPopes>
* 6b55b9c - feat(`interpreter`): add hash to bytecode (#628) (5 weeks ago) <evalir>
* 84a5e97 - chore(interpreter): use `let else` (#629) (5 weeks ago) <DaniPopes>
* e9d96cd - chore(interpreter): improve dummy host (#631) (5 weeks ago) <DaniPopes>
* 2054293 - chore: misc improvements (#633) (5 weeks ago) <DaniPopes>
* 68820da - feat(state): Block hash cache and overrides (#621) (5 weeks ago) <rakita>
* eb6a9f0 - Revert "feat: alloy migration (#535)" (#616) (6 weeks ago) <rakita>
* c1bad0d - chore: spell check (#615) (6 weeks ago) <Roman Krasiuk>
* f95b7a4 - feat: alloy migration (#535) (6 weeks ago) <DaniPopes>
* bc4d203 - feat: remove unneccesary var and if branch in gas calc (#592) (7 weeks ago) <bemevolent>
* ef57a46 - feat: State with account status (#499) (7 weeks ago) <rakita>
* 157ef36 - feat: introduce initcode size limit check taking config into account (#587) (7 weeks ago) <evalir>
* 12558c5 - fix: fix mcopy memory expansion. Add eth tests to ci (#586) (7 weeks ago) <rakita>
* 06b1f6b - feat: EIP-1153 Transient storage opcodes (#546) (8 weeks ago) <Mark Tyneway>
* c6c5e88 - make calc public  (#575) (8 weeks ago) <BrazilRaw>
* 0a739e4 - fix(interpreter): mcopy call order (#570) (8 weeks ago) <DaniPopes>
* 30bfa73 - fix(doc): Inline documentation of re-exports (#560) (9 weeks ago) <Yiannis Marangos>
* 36de35b - feat: Rename all SHA3 opcodes to KECCAK256 (#514) (3 months ago) <Tung Bui (Leo)>
* 10f81ba - optimize stack usage for recursive `call` and `create` programs (#522) (3 months ago) <Valentin Mihov>
* c153428 - feat(cancun): EIP-5656: MCOPY - Memory copying instruction (#528) (3 months ago) <Waylon Jepsen>
* 51072e6 - consume all gas on invalid opcode (#500) (3 months ago) <teddav>
* ccd0298 - feat: add Memory::into_data (#516) (3 months ago) <Matthias Seitz>
* 69f417f - feat: simplify BYTE opcode (#512) (4 months ago) <teddav>
* c54f079 - fix: replace SHA3 with KECCAK256 opcode name (#511) (4 months ago) <Matthias Seitz>
* f8ff6b3 - feat: separate initial checks (#486) (5 months ago) <rakita>
* 6057cc2 - chore: refactor interpreter run and remove static flag (#481) (5 months ago) <rakita>


# v1.1.2
date: 03.05.2023

* 08091e1 - fix: compile errors for features (#467) (13 days ago) <rakita>

# v1.1.1
date: 14.04.2023

Added back utility function:
* 7d9b38a - [Interpreter]: Add back `spec_gas_opcode` (#446) (9 days ago) <Enrique Ortiz>

# v1.1.0
date: 04.04.2023

Biggest changes are Shanghai support 08ce847 and removal of gas blocks f91d5f9.

Changelog:
* c2ee8ff - add feature for ignoring base fee check (#436) (6 days ago) <Dan Cline>
* 0eff6a7 - Fix panic! message (#431) (2 weeks ago) <David Kulman>
* d0038e3 - chore(deps): bump arbitrary from 1.2.3 to 1.3.0 (#428) (2 weeks ago) <dependabot[bot]>
* dd0e227 - feat: Add all internals results to Halt (#413) (4 weeks ago) <rakita>
* d8dc652 - fix(interpreter): halt on CreateInitcodeSizeLimit (#412) (4 weeks ago) <Roman Krasiuk>
* a193d79 - chore: enabled primtive default feature in precompile (#409) (4 weeks ago) <Matthias Seitz>
* 1720729 - chore: add display impl for Opcode (#406) (4 weeks ago) <Matthias Seitz>
* 33bf8a8 - feat: use singular bytes for the jumpmap (#402) (4 weeks ago) <Bjerg>
* 394e8e9 - feat: extend SuccessOrHalt (#405) (4 weeks ago) <Matthias Seitz>
* f91d5f9 - refactor: remove gas blocks (#391) (5 weeks ago) <Bjerg>
* a8ae3f4 - fix: using pop_top instead of pop in eval_exp (#379) (7 weeks ago) <flyq>
* 08ce847 - feat(Shanghai): All EIPs: push0, warm coinbase, limit/measure initcode (#376) (7 weeks ago) <rakita>
* 6710511 - add no_std to primitives (#366) (7 weeks ago) <rakita>
* 1fca102 - chore(deps): bump proptest from 1.0.0 to 1.1.0 (#358) (8 weeks ago) <dependabot[bot]>
* 9b663bb - feat: Different OutOfGas Error types (#354) (9 weeks ago) <Chirag Baghasingh>

# v1.0.0
date: 29.01.2023

Interpreter was extracted from main revm crate at the revm v3.0.0 version.