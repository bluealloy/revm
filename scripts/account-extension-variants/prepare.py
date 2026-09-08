#!/usr/bin/env python3
"""Install an experimental representation in a clean, detached revm worktree.

Usage: prepare.py WORKTREE bytes|wrapped|thinarc|ecobytes|arc
This deliberately rewrites only the named experimental checkout, not the PR source.
"""
from pathlib import Path
import shutil
import subprocess
import sys

root = Path(sys.argv[1]).resolve()
variant = sys.argv[2]
here = Path(__file__).resolve().parent
repo = here.parents[1]
assert variant in ("bytes", "wrapped", "thinarc", "ecobytes", "arc")
assert root != repo
assert subprocess.check_output(["git", "status", "--porcelain"], cwd=root, text=True) == ""
assert subprocess.run(["git", "symbolic-ref", "-q", "HEAD"], cwd=root, capture_output=True).returncode != 0

def replace(path, old, new):
    path = root / path
    text = path.read_text()
    assert old in text, (path, old)
    path.write_text(text.replace(old, new))

# Identical dependency/features in every build, including the untouched Bytes control.
replace("crates/state/Cargo.toml", "[dependencies]", '[dependencies]\ntriomphe = { version = "=0.1.16", default-features = false }\necow = { version = "=0.3.1", default-features = false }')
replace("crates/state/src/lib.rs", "mod account_info;", "use triomphe as _;\nuse ecow as _;\nmod account_info;")
shutil.copyfile(repo / "Cargo.lock", root / "Cargo.lock")
shutil.copyfile(repo / "bins/revme/benches/account_extension.rs", root / "bins/revme/benches/account_extension.rs")
replace("bins/revme/Cargo.toml", '[[bench]]\nname = "evm"', '[[bench]]\nname = "account_extension_storage"\nharness = false\n\n[[bench]]\nname = "evm"')
storage = (here / "storage.rs").read_text()
storage = storage.replace("EXTENSION_IMPORT", "use revm::primitives::Bytes as Extension;" if variant == "bytes" else "use revm::state::AccountExtension as Extension;")
(root / "bins/revme/benches/account_extension_storage.rs").write_text(storage)
allocations = (here / "allocations.rs").read_text().replace("EXTENSION_IMPORT", "use revm::primitives::Bytes as Extension;" if variant == "bytes" else "use revm::state::AccountExtension as Extension;")
(root / "bins/revme/examples/account_extension_allocations.rs").write_text(allocations)
subcalls = (repo / "bins/revme/src/cmd/bench/subcall.rs").read_text()
subcalls = subcalls.replace("pub fn run(criterion: &mut Criterion)", "pub fn run(criterion: &mut Criterion, payload_len: usize)")
subcalls = subcalls.replace("..Default::default()\n            },", "..Default::default()\n            }.with_extension(revm::primitives::Bytes::from(vec![42; payload_len])),")
for case in ["transfer_1wei", "same_account", "nested"]:
    subcalls = subcalls.replace(f'"subcall_1000_{case}"', f'&format!("subcall_1000_{case}/payload={{payload_len}}")')
subcalls = subcalls.replace("        criterion.bench_function", "        assert!(evm.transact_one(tx.clone()).unwrap().is_success());\n        criterion.bench_function")
subcalls += '\nfn benches(c: &mut Criterion) { run(c, 0); run(c, 32); }\ncriterion::criterion_group!(group, benches);\ncriterion::criterion_main!(group);\n'
(root / "bins/revme/benches/account_extension_subcalls.rs").write_text(subcalls)
replace("bins/revme/Cargo.toml", '[[bench]]\nname = "evm"', '[[bench]]\nname = "account_extension_subcalls"\nharness = false\n\n[[bench]]\nname = "evm"')

if variant != "bytes":
    options = {
        "wrapped": ("Bytes", "Bytes::new()", "Self(Bytes::copy_from_slice(bytes))", "self.0.is_empty()", "self.0.as_ref()", "Self(bytes)", "Self(bytes.into())"),
        "thinarc": ("Option<triomphe::ThinArc<(), u8>>", "None", "Self((!bytes.is_empty()).then(|| triomphe::ThinArc::from_header_and_slice((), bytes)))", "self.0.is_none()", "self.0.as_ref().map_or(&[], |a| &a.slice)", "Self::copy_from_slice(&bytes)", "Self::copy_from_slice(&bytes)"),
        "ecobytes": ("ecow::EcoBytes", "ecow::EcoBytes::new()", "Self(ecow::EcoBytes::from(bytes))", "self.0.is_empty()", "self.0.as_ref()", "Self::copy_from_slice(&bytes)", "Self::copy_from_slice(&bytes)"),
        "arc": ("Option<std::sync::Arc<[u8]>>", "None", "Self((!bytes.is_empty()).then(|| std::sync::Arc::from(bytes)))", "self.0.is_none()", "self.0.as_deref().unwrap_or(&[])", "Self::copy_from_slice(&bytes)", "Self::copy_from_slice(&bytes)"),
    }
    source = (here / "extension.rs").read_text()
    for key, value in zip(["STORAGE_TYPE", "EMPTY_STORAGE", "COPY_STORAGE", "EMPTY_TEST", "SLICE_ACCESS", "FROM_BYTES", "FROM_VEC"], options[variant]):
        source = source.replace(key, value)
    (root / "crates/state/src/account_extension.rs").write_text(source)
    replace("crates/state/src/lib.rs", "mod account_info;", "mod account_info;\nmod account_extension;\npub use account_extension::AccountExtension;")
    replace("crates/state/src/account_info.rs", "use bytecode::Bytecode;", "use bytecode::Bytecode;\nuse crate::AccountExtension;")
    replace("crates/state/src/account_info.rs", "pub extension: Bytes,", "pub extension: AccountExtension,")
    replace("crates/state/src/account_info.rs", "extension: Bytes::new()", "extension: AccountExtension::new()")
    replace("crates/state/src/account_info.rs", "self.extension = extension;", "self.extension = extension.into();")
    replace("crates/state/src/account_info.rs", "pub const fn set_extension(&mut self, extension: Bytes) -> Bytes {", "pub fn set_extension(&mut self, extension: Bytes) -> AccountExtension {")
    replace("crates/state/src/account_info.rs", "core::mem::replace(&mut self.extension, extension)", "core::mem::replace(&mut self.extension, extension.into())")
    replace("crates/state/src/bal/account.rs", "use primitives::{Address, Bytes,", "use crate::AccountExtension;\nuse primitives::{Address,")
    replace("crates/state/src/bal/account.rs", "BalWrites<Bytes>", "BalWrites<AccountExtension>")
