//! EVM gasometer.

//#![deny(warnings)]
//#![forbid(unsafe_code, unused_variables)]
//#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use super::{calc, constants};
use crate::{error::ExitError, machine::Stack, opcode::OpCode, ExtHandler};
use alloc::vec::Vec;
use core::cmp::max;
use primitive_types::{H160, H256, U256};
use std::marker::PhantomData;

use super::Spec;

macro_rules! try_or_fail {
    ( $inner:expr, $e:expr ) => {
        match $e {
            Ok(value) => value,
            Err(e) => {
                $inner = Err(e.clone());
                return Err(e);
            }
        }
    };
}

#[derive(Debug, Copy, Clone)]
pub struct Snapshot {
    pub gas_limit: u64,
    pub memory_gas: u64,
    pub used_gas: u64,
    pub refunded_gas: i64,
}

/// EVM gasometer.
#[derive(Clone)]
pub struct Gasometer<SPEC: Spec> {
    gas_limit: u64,
    gas: Inner<SPEC>,
    status: Result<(), ExitError>,
    phantom: PhantomData<SPEC>,
}

impl<SPEC: Spec> Gasometer<SPEC> {
    /// Create a new gasometer with given gas limit and SPEC::
    pub fn new(gas_limit: u64) -> Self {
        Self {
            gas_limit,
            gas: Inner {
                memory_gas: 0,
                used_gas: 0,
                refunded_gas: 0,
                phantomdata: PhantomData,
            },
            status: Ok(()),
            phantom: PhantomData,
        }
    }

    #[inline]
    /// Returns the numerical gas cost value.
    pub fn gas_cost(&self, cost: GasCost, gas: u64) -> Result<u64, ExitError> {
        self.status?;
        self.gas.gas_cost(cost, gas)
    }

    #[inline]
    /// Remaining gas.
    pub fn gas(&self) -> u64 {
        match self.status {
            Ok(_) => self.gas_limit - self.gas.used_gas - self.gas.memory_gas,
            Err(_) => 0,
        }
    }

    #[inline]
    /// Total used gas.
    pub fn total_used_gas(&self) -> u64 {
        match self.status {
            Ok(_) => self.gas.used_gas + self.gas.memory_gas,
            Err(_) => self.gas_limit,
        }
    }

    #[inline]
    /// Refunded gas.
    pub fn refunded_gas(&self) -> i64 {
        match self.status {
            Ok(_) => self.gas.refunded_gas,
            Err(_) => 0,
        }
    }

    /// Explictly fail the gasometer with out of gas. Return `OutOfGas` error.
    pub fn fail(&mut self) -> ExitError {
        self.status = Err(ExitError::OutOfGas);
        ExitError::OutOfGas
    }

    #[inline]
    /// Record an explict cost.
    pub fn record_cost<const GAS_TRACE: bool>(&mut self, cost: u64) -> Result<(), ExitError> {
        if GAS_TRACE {
            // event!(RecordCost {
            // 	cost,
            // 	snapshot: self.snapshot()?,
            // });
        }

        let all_gas_cost = self.total_used_gas() + cost;
        if self.gas_limit < all_gas_cost {
            self.status = Err(ExitError::OutOfGas);
            return Err(ExitError::OutOfGas);
        }

        self.status?;
        self.gas.used_gas += cost;
        Ok(())
    }

    #[inline]
    /// Record an explict refund.
    pub fn record_refund<const GAS_TRACE: bool>(&mut self, refund: i64) -> Result<(), ExitError> {
        if GAS_TRACE {
            // event!(RecordRefund {
            // 	refund,
            // 	snapshot: self.snapshot()?,
            // });
        }
        self.status?;
        self.gas.refunded_gas += refund;
        Ok(())
    }

    #[inline]
    /// Record `CREATE` code deposit.
    pub fn record_deposit<const GAS_TRACE: bool>(&mut self, len: usize) -> Result<(), ExitError> {
        let cost = len as u64 * constants::G_CODEDEPOSIT;
        self.record_cost::<GAS_TRACE>(cost)
    }

    /// Record opcode gas cost.
    pub fn record_dynamic_cost<const GAS_TRACE: bool>(
        &mut self,
        cost: GasCost,
        memory: Option<MemoryCost>,
    ) -> Result<(), ExitError> {
        let gas = self.gas();

        self.status?;
        let memory_gas = match memory {
            Some(memory) => try_or_fail!(self.status, self.gas.memory_gas(memory)),
            None => self.gas.memory_gas,
        };
        let gas_cost = try_or_fail!(self.status, self.gas.gas_cost(cost, gas));
        let gas_refund = self.gas.gas_refund(cost);
        let used_gas = self.gas.used_gas;
        if GAS_TRACE {
            // event!(RecordDynamicCost {
            // 	gas_cost,
            // 	memory_gas,
            // 	gas_refund,
            // 	snapshot: self.snapshot()?,
            // });
        }

        let all_gas_cost = memory_gas + used_gas + gas_cost;
        if self.gas_limit < all_gas_cost {
            self.status = Err(ExitError::OutOfGas);
            return Err(ExitError::OutOfGas);
        }

        let after_gas = self.gas_limit - all_gas_cost;
        try_or_fail!(self.status, self.gas.extra_check(cost, after_gas));

        self.gas.used_gas += gas_cost;
        self.gas.memory_gas = memory_gas;
        self.gas.refunded_gas += gas_refund;

        Ok(())
    }

    #[inline]
    /// Record opcode stipend.
    pub fn record_stipend<const GAS_TRACE: bool>(&mut self, stipend: u64) -> Result<(), ExitError> {
        if GAS_TRACE {
            // event!(RecordStipend {
            // 	stipend,
            // 	snapshot: self.snapshot()?,
            // });
        }

        self.gas.used_gas -= stipend;
        Ok(())
    }

    /// Record transaction cost.
    pub fn record_transaction<const GAS_TRACE: bool>(
        &mut self,
        cost: TransactionCost,
    ) -> Result<(), ExitError> {
        let gas_cost = match cost {
            TransactionCost::Call {
                zero_data_len,
                non_zero_data_len,
                access_list_address_len,
                access_list_storage_len,
            } => {
                SPEC::gas_transaction_call
                    + zero_data_len as u64 * SPEC::gas_transaction_zero_data
                    + non_zero_data_len as u64 * SPEC::gas_transaction_non_zero_data
                    + access_list_address_len as u64 * SPEC::gas_access_list_address
                    + access_list_storage_len as u64 * SPEC::gas_access_list_storage_key
            }
            TransactionCost::Create {
                zero_data_len,
                non_zero_data_len,
                access_list_address_len,
                access_list_storage_len,
            } => {
                SPEC::gas_transaction_create
                    + zero_data_len as u64 * SPEC::gas_transaction_zero_data
                    + non_zero_data_len as u64 * SPEC::gas_transaction_non_zero_data
                    + access_list_address_len as u64 * SPEC::gas_access_list_address
                    + access_list_storage_len as u64 * SPEC::gas_access_list_storage_key
            }
        };
        if GAS_TRACE {
            // event!(RecordTransaction {
            // 	cost: gas_cost,
            // 	snapshot: self.snapshot()?,
            // });
        }

        if self.gas() < gas_cost {
            self.status = Err(ExitError::OutOfGas);
            return Err(ExitError::OutOfGas);
        }

        self.gas.used_gas += gas_cost;
        Ok(())
    }

    pub fn snapshot(&self) -> Result<Snapshot, ExitError> {
        self.status?;
        let inner = self.gas.clone();
        Ok(Snapshot {
            gas_limit: self.gas_limit,
            memory_gas: inner.memory_gas,
            used_gas: inner.used_gas,
            refunded_gas: inner.refunded_gas,
        })
    }
}

/// Calculate the call transaction cost.
pub fn call_transaction_cost(data: &[u8], access_list: &[(H160, Vec<H256>)]) -> TransactionCost {
    let zero_data_len = data.iter().filter(|v| **v == 0).count();
    let non_zero_data_len = data.len() - zero_data_len;
    let (access_list_address_len, access_list_storage_len) = count_access_list(access_list);

    TransactionCost::Call {
        zero_data_len,
        non_zero_data_len,
        access_list_address_len,
        access_list_storage_len,
    }
}

/// Calculate the create transaction cost.
pub fn create_transaction_cost(data: &[u8], access_list: &[(H160, Vec<H256>)]) -> TransactionCost {
    let zero_data_len = data.iter().filter(|v| **v == 0).count();
    let non_zero_data_len = data.len() - zero_data_len;
    let (access_list_address_len, access_list_storage_len) = count_access_list(access_list);

    TransactionCost::Create {
        zero_data_len,
        non_zero_data_len,
        access_list_address_len,
        access_list_storage_len,
    }
}

/// Counts the number of addresses and storage keys in the access list
fn count_access_list(access_list: &[(H160, Vec<H256>)]) -> (usize, usize) {
    let access_list_address_len = access_list.len();
    let access_list_storage_len = access_list.iter().map(|(_, keys)| keys.len()).sum();

    (access_list_address_len, access_list_storage_len)
}

#[inline]
pub fn static_opcode_cost(opcode: OpCode) -> Option<u64> {
    static TABLE: [Option<u64>; 256] = {
        let mut table = [None; 256];

        table[OpCode::STOP.as_usize()] = Some(constants::G_ZERO);
        table[OpCode::CALLDATASIZE.as_usize()] = Some(constants::G_BASE);
        table[OpCode::CODESIZE.as_usize()] = Some(constants::G_BASE);
        table[OpCode::POP.as_usize()] = Some(constants::G_BASE);
        table[OpCode::PC.as_usize()] = Some(constants::G_BASE);
        table[OpCode::MSIZE.as_usize()] = Some(constants::G_BASE);

        table[OpCode::ADDRESS.as_usize()] = Some(constants::G_BASE);
        table[OpCode::ORIGIN.as_usize()] = Some(constants::G_BASE);
        table[OpCode::CALLER.as_usize()] = Some(constants::G_BASE);
        table[OpCode::CALLVALUE.as_usize()] = Some(constants::G_BASE);
        table[OpCode::COINBASE.as_usize()] = Some(constants::G_BASE);
        table[OpCode::TIMESTAMP.as_usize()] = Some(constants::G_BASE);
        table[OpCode::NUMBER.as_usize()] = Some(constants::G_BASE);
        table[OpCode::DIFFICULTY.as_usize()] = Some(constants::G_BASE);
        table[OpCode::GASLIMIT.as_usize()] = Some(constants::G_BASE);
        table[OpCode::GASPRICE.as_usize()] = Some(constants::G_BASE);
        table[OpCode::GAS.as_usize()] = Some(constants::G_BASE);

        table[OpCode::ADD.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::SUB.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::NOT.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::LT.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::GT.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::SLT.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::SGT.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::EQ.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::ISZERO.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::AND.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::OR.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::XOR.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::BYTE.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::CALLDATALOAD.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::PUSH1.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::PUSH2.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::PUSH3.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::PUSH4.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::PUSH5.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::PUSH6.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::PUSH7.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::PUSH8.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::PUSH9.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::PUSH10.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::PUSH11.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::PUSH12.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::PUSH13.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::PUSH14.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::PUSH15.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::PUSH16.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::PUSH17.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::PUSH18.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::PUSH19.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::PUSH20.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::PUSH21.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::PUSH22.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::PUSH23.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::PUSH24.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::PUSH25.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::PUSH26.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::PUSH27.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::PUSH28.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::PUSH29.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::PUSH30.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::PUSH31.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::PUSH32.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::DUP1.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::DUP2.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::DUP3.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::DUP4.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::DUP5.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::DUP6.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::DUP7.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::DUP8.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::DUP9.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::DUP10.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::DUP11.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::DUP12.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::DUP13.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::DUP14.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::DUP15.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::DUP16.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::SWAP1.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::SWAP2.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::SWAP3.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::SWAP4.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::SWAP5.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::SWAP6.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::SWAP7.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::SWAP8.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::SWAP9.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::SWAP10.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::SWAP11.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::SWAP12.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::SWAP13.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::SWAP14.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::SWAP15.as_usize()] = Some(constants::G_VERYLOW);
        table[OpCode::SWAP16.as_usize()] = Some(constants::G_VERYLOW);

        table[OpCode::MUL.as_usize()] = Some(constants::G_LOW);
        table[OpCode::DIV.as_usize()] = Some(constants::G_LOW);
        table[OpCode::SDIV.as_usize()] = Some(constants::G_LOW);
        table[OpCode::MOD.as_usize()] = Some(constants::G_LOW);
        table[OpCode::SMOD.as_usize()] = Some(constants::G_LOW);
        table[OpCode::SIGNEXTEND.as_usize()] = Some(constants::G_LOW);

        table[OpCode::ADDMOD.as_usize()] = Some(constants::G_MID);
        table[OpCode::MULMOD.as_usize()] = Some(constants::G_MID);
        table[OpCode::JUMP.as_usize()] = Some(constants::G_MID);

        table[OpCode::JUMPI.as_usize()] = Some(constants::G_HIGH);
        table[OpCode::JUMPDEST.as_usize()] = Some(constants::G_JUMPDEST);

        table
    };

    TABLE[opcode.as_usize()]
}

/// Calculate the opcode cost.
#[allow(clippy::nonminimal_bool)]
pub fn dynamic_opcode_cost<H: ExtHandler, SPEC: Spec>(
    address: H160,
    opcode: OpCode,
    stack: &Stack,
    is_static: bool,
    handler: &mut H,
) -> Result<(GasCost, StorageTarget, Option<MemoryCost>), ExitError> {
    let mut storage_target = StorageTarget::None;
    let gas_cost = match opcode {
        OpCode::RETURN => GasCost::Zero,

        OpCode::MLOAD | OpCode::MSTORE | OpCode::MSTORE8 => GasCost::VeryLow,

        OpCode::REVERT if SPEC::has_revert => GasCost::Zero,
        OpCode::REVERT => GasCost::Invalid,

        OpCode::CHAINID if SPEC::has_chain_id => GasCost::Base,
        OpCode::CHAINID => GasCost::Invalid,

        OpCode::SHL | OpCode::SHR | OpCode::SAR if SPEC::has_bitwise_shifting => GasCost::VeryLow,
        OpCode::SHL | OpCode::SHR | OpCode::SAR => GasCost::Invalid,

        OpCode::SELFBALANCE if SPEC::has_self_balance => GasCost::Low,
        OpCode::SELFBALANCE => GasCost::Invalid,

        OpCode::EXTCODESIZE => {
            let target = stack.peek(0)?.into();
            storage_target = StorageTarget::Address(target);
            GasCost::ExtCodeSize {
                target_is_cold: handler.is_cold(target),
            }
        }
        OpCode::BALANCE => {
            let target = stack.peek(0)?.into();
            storage_target = StorageTarget::Address(target);
            GasCost::Balance {
                target_is_cold: handler.is_cold(target),
            }
        }
        OpCode::BLOCKHASH => GasCost::BlockHash,

        OpCode::EXTCODEHASH if SPEC::has_ext_code_hash => {
            let target = stack.peek(0)?.into();
            storage_target = StorageTarget::Address(target);
            GasCost::ExtCodeHash {
                target_is_cold: handler.is_cold(target),
            }
        }
        OpCode::EXTCODEHASH => GasCost::Invalid,

        OpCode::CALLCODE => {
            let target = stack.peek(1)?.into();
            storage_target = StorageTarget::Address(target);
            GasCost::CallCode {
                value: U256::from_big_endian(&stack.peek(2)?[..]),
                gas: U256::from_big_endian(&stack.peek(0)?[..]),
                target_is_cold: handler.is_cold(target),
                target_exists: true, //handler.exists(target)
            }
        }
        OpCode::STATICCALL => {
            let target = stack.peek(1)?.into();
            storage_target = StorageTarget::Address(target);
            GasCost::StaticCall {
                gas: U256::from_big_endian(&stack.peek(0)?[..]),
                target_is_cold: handler.is_cold(target),
                target_exists: true, //handler.exists(target)
            }
        }
        OpCode::SHA3 => GasCost::Sha3 {
            len: U256::from_big_endian(&stack.peek(1)?[..]),
        },
        OpCode::EXTCODECOPY => {
            let target = stack.peek(0)?.into();
            storage_target = StorageTarget::Address(target);
            GasCost::ExtCodeCopy {
                target_is_cold: handler.is_cold(target),
                len: U256::from_big_endian(&stack.peek(3)?[..]),
            }
        }
        OpCode::CALLDATACOPY | OpCode::CODECOPY => GasCost::VeryLowCopy {
            len: U256::from_big_endian(&stack.peek(2)?[..]),
        },
        OpCode::EXP => GasCost::Exp {
            power: U256::from_big_endian(&stack.peek(1)?[..]),
        },
        OpCode::SLOAD => {
            let index = stack.peek(0)?;
            storage_target = StorageTarget::Slot(address, index);
            GasCost::SLoad {
                target_is_cold: handler.is_cold_storage(address, index),
            }
        }

        OpCode::DELEGATECALL if SPEC::has_delegate_call => {
            let target = stack.peek(1)?.into();
            storage_target = StorageTarget::Address(target);
            GasCost::DelegateCall {
                gas: U256::from_big_endian(&stack.peek(0)?[..]),
                target_is_cold: handler.is_cold(target),
                target_exists: true, //handler.exists(target)
            }
        }
        OpCode::DELEGATECALL => GasCost::Invalid,

        OpCode::RETURNDATASIZE if SPEC::has_return_data => GasCost::Base,
        OpCode::RETURNDATACOPY if SPEC::has_return_data => GasCost::VeryLowCopy {
            len: U256::from_big_endian(&stack.peek(2)?[..]),
        },
        OpCode::RETURNDATASIZE | OpCode::RETURNDATACOPY => GasCost::Invalid,

        OpCode::SSTORE if !is_static => {
            let index = stack.peek(0)?;
            let value = stack.peek(1)?;
            storage_target = StorageTarget::Slot(address, index);

            GasCost::SStore {
                original: handler.original_storage(address, index),
                current: handler.sload(address, index).0,
                new: value,
                target_is_cold: handler.is_cold_storage(address, index),
            }
        }
        OpCode::LOG0 if !is_static => GasCost::Log {
            n: 0,
            len: U256::from_big_endian(&stack.peek(1)?[..]),
        },
        OpCode::LOG1 if !is_static => GasCost::Log {
            n: 1,
            len: U256::from_big_endian(&stack.peek(1)?[..]),
        },
        OpCode::LOG2 if !is_static => GasCost::Log {
            n: 2,
            len: U256::from_big_endian(&stack.peek(1)?[..]),
        },
        OpCode::LOG3 if !is_static => GasCost::Log {
            n: 3,
            len: U256::from_big_endian(&stack.peek(1)?[..]),
        },
        OpCode::LOG4 if !is_static => GasCost::Log {
            n: 4,
            len: U256::from_big_endian(&stack.peek(1)?[..]),
        },
        OpCode::CREATE if !is_static => GasCost::Create,
        OpCode::CREATE2 if !is_static && SPEC::has_create2 => GasCost::Create2 {
            len: U256::from_big_endian(&stack.peek(2)?[..]),
        },
        OpCode::SUICIDE if !is_static => {
            let target = stack.peek(0)?.into();
            storage_target = StorageTarget::Address(target);
            GasCost::Suicide {
                value: handler.balance(address).0,
                target_is_cold: handler.is_cold(target),
                target_exists: true,    //handler.exists(target)
                already_removed: false, //handler.deleted(address),
            }
        }
        OpCode::CALL
            if !is_static
                || (is_static && U256::from_big_endian(&stack.peek(2)?[..]) == U256::zero()) =>
        {
            let target = stack.peek(1)?.into();
            storage_target = StorageTarget::Address(target);
            GasCost::Call {
                value: U256::from_big_endian(&stack.peek(2)?[..]),
                gas: U256::from_big_endian(&stack.peek(0)?[..]),
                target_is_cold: handler.is_cold(target),
                target_exists: true, //handler.exists(target)
            }
        }

        _ => GasCost::Invalid,
    };

    let memory_cost = match opcode {
        OpCode::SHA3
        | OpCode::RETURN
        | OpCode::REVERT
        | OpCode::LOG0
        | OpCode::LOG1
        | OpCode::LOG2
        | OpCode::LOG3
        | OpCode::LOG4 => Some(MemoryCost {
            offset: U256::from_big_endian(&stack.peek(0)?[..]),
            len: U256::from_big_endian(&stack.peek(1)?[..]),
        }),

        OpCode::CODECOPY | OpCode::CALLDATACOPY | OpCode::RETURNDATACOPY => Some(MemoryCost {
            offset: U256::from_big_endian(&stack.peek(0)?[..]),
            len: U256::from_big_endian(&stack.peek(2)?[..]),
        }),

        OpCode::EXTCODECOPY => Some(MemoryCost {
            offset: U256::from_big_endian(&stack.peek(1)?[..]),
            len: U256::from_big_endian(&stack.peek(3)?[..]),
        }),

        OpCode::MLOAD | OpCode::MSTORE => Some(MemoryCost {
            offset: U256::from_big_endian(&stack.peek(0)?[..]),
            len: U256::from(32),
        }),

        OpCode::MSTORE8 => Some(MemoryCost {
            offset: U256::from_big_endian(&stack.peek(0)?[..]),
            len: U256::from(1),
        }),

        OpCode::CREATE | OpCode::CREATE2 => Some(MemoryCost {
            offset: U256::from_big_endian(&stack.peek(1)?[..]),
            len: U256::from_big_endian(&stack.peek(2)?[..]),
        }),

        OpCode::CALL | OpCode::CALLCODE => Some(
            MemoryCost {
                offset: U256::from_big_endian(&stack.peek(3)?[..]),
                len: U256::from_big_endian(&stack.peek(4)?[..]),
            }
            .join(MemoryCost {
                offset: U256::from_big_endian(&stack.peek(5)?[..]),
                len: U256::from_big_endian(&stack.peek(6)?[..]),
            }),
        ),

        OpCode::DELEGATECALL | OpCode::STATICCALL => Some(
            MemoryCost {
                offset: U256::from_big_endian(&stack.peek(2)?[..]),
                len: U256::from_big_endian(&stack.peek(3)?[..]),
            }
            .join(MemoryCost {
                offset: U256::from_big_endian(&stack.peek(4)?[..]),
                len: U256::from_big_endian(&stack.peek(5)?[..]),
            }),
        ),

        _ => None,
    };

    Ok((gas_cost, storage_target, memory_cost))
}

/// Holds the gas consumption for a Gasometer instance.
#[derive(Clone)]
struct Inner<SPEC: Spec> {
    memory_gas: u64,
    used_gas: u64,
    refunded_gas: i64,
    phantomdata: PhantomData<SPEC>,
}

impl<SPEC: Spec> Inner<SPEC> {
    fn memory_gas(&self, memory: MemoryCost) -> Result<u64, ExitError> {
        let from = memory.offset;
        let len = memory.len;

        if len == U256::zero() {
            return Ok(self.memory_gas);
        }

        let end = from.checked_add(len).ok_or(ExitError::OutOfGas)?;

        if end > U256::from(usize::MAX) {
            return Err(ExitError::OutOfGas);
        }
        let end = end.as_usize();

        let rem = end % 32;
        let new = if rem == 0 { end / 32 } else { end / 32 + 1 };

        Ok(max(self.memory_gas, super::calc::memory_gas(new)?))
    }

    fn extra_check(&self, cost: GasCost, after_gas: u64) -> Result<(), ExitError> {
        match cost {
            GasCost::Call { gas, .. } => calc::call_extra_check::<SPEC>(gas, after_gas),
            GasCost::CallCode { gas, .. } => calc::call_extra_check::<SPEC>(gas, after_gas),
            GasCost::DelegateCall { gas, .. } => calc::call_extra_check::<SPEC>(gas, after_gas),
            GasCost::StaticCall { gas, .. } => calc::call_extra_check::<SPEC>(gas, after_gas),
            _ => Ok(()),
        }
    }

    /// Returns the gas cost numerical value.
    fn gas_cost(&self, cost: GasCost, gas: u64) -> Result<u64, ExitError> {
        Ok(match cost {
            GasCost::Call {
                value,
                target_is_cold,
                target_exists,
                ..
            } => calc::call_cost::<SPEC>(value, target_is_cold, true, true, !target_exists),
            GasCost::CallCode {
                value,
                target_is_cold,
                target_exists,
                ..
            } => calc::call_cost::<SPEC>(value, target_is_cold, true, false, !target_exists),
            GasCost::DelegateCall {
                target_is_cold,
                target_exists,
                ..
            } => {
                calc::call_cost::<SPEC>(U256::zero(), target_is_cold, false, false, !target_exists)
            }
            GasCost::StaticCall {
                target_is_cold,
                target_exists,
                ..
            } => calc::call_cost::<SPEC>(U256::zero(), target_is_cold, false, true, !target_exists),

            GasCost::Suicide {
                value,
                target_is_cold,
                target_exists,
                ..
            } => calc::suicide_cost::<SPEC>(value, target_is_cold, target_exists),
            GasCost::SStore { .. } if SPEC::estimate => SPEC::gas_sstore_set,
            GasCost::SStore {
                original,
                current,
                new,
                target_is_cold,
            } => calc::sstore_cost::<SPEC>(original, current, new, gas, target_is_cold)?,

            GasCost::Sha3 { len } => calc::sha3_cost(len)?,
            GasCost::Log { n, len } => calc::log_cost(n, len)?,
            GasCost::VeryLowCopy { len } => calc::verylowcopy_cost(len)?,
            GasCost::Exp { power } => calc::exp_cost::<SPEC>(power)?,
            GasCost::Create => constants::G_CREATE,
            GasCost::Create2 { len } => calc::create2_cost(len)?,
            GasCost::SLoad { target_is_cold } => calc::sload_cost::<SPEC>(target_is_cold),

            GasCost::Zero => constants::G_ZERO,
            GasCost::Base => constants::G_BASE,
            GasCost::VeryLow => constants::G_VERYLOW,
            GasCost::Low => constants::G_LOW,
            GasCost::Invalid => return Err(ExitError::OutOfGas),

            GasCost::ExtCodeSize { target_is_cold } => {
                calc::address_access_cost::<SPEC>(target_is_cold, SPEC::gas_ext_code)
            }
            GasCost::ExtCodeCopy {
                target_is_cold,
                len,
            } => calc::extcodecopy_cost::<SPEC>(len, target_is_cold)?,
            GasCost::Balance { target_is_cold } => {
                calc::address_access_cost::<SPEC>(target_is_cold, SPEC::gas_balance)
            }
            GasCost::BlockHash => constants::G_BLOCKHASH,
            GasCost::ExtCodeHash { target_is_cold } => {
                calc::address_access_cost::<SPEC>(target_is_cold, SPEC::gas_ext_code_hash)
            }
        })
    }

    fn gas_refund(&self, cost: GasCost) -> i64 {
        match cost {
            _ if SPEC::estimate => 0,

            GasCost::SStore {
                original,
                current,
                new,
                ..
            } => calc::sstore_refund::<SPEC>(original, current, new),
            GasCost::Suicide {
                already_removed, ..
            } => calc::suicide_refund(already_removed),
            _ => 0,
        }
    }
}

/// Gas cost.
#[derive(Debug, Clone, Copy)]
pub enum GasCost {
    /// Zero gas cost.
    Zero,
    /// Base gas cost.
    Base,
    /// Very low gas cost.
    VeryLow,
    /// Low gas cost.
    Low,
    /// Fail the gasometer.
    Invalid,

    /// Gas cost for `EXTCODESIZE`.
    ExtCodeSize {
        /// True if address has not been previously accessed in this transaction
        target_is_cold: bool,
    },
    /// Gas cost for `BALANCE`.
    Balance {
        /// True if address has not been previously accessed in this transaction
        target_is_cold: bool,
    },
    /// Gas cost for `BLOCKHASH`.
    BlockHash,
    /// Gas cost for `EXTBLOCKHASH`.
    ExtCodeHash {
        /// True if address has not been previously accessed in this transaction
        target_is_cold: bool,
    },

    /// Gas cost for `CALL`.
    Call {
        /// Call value.
        value: U256,
        /// Call gas.
        gas: U256,
        /// True if target has not been previously accessed in this transaction
        target_is_cold: bool,
        /// Whether the target exists.
        target_exists: bool,
    },
    /// Gas cost for `CALLCODE.
    CallCode {
        /// Call value.
        value: U256,
        /// Call gas.
        gas: U256,
        /// True if target has not been previously accessed in this transaction
        target_is_cold: bool,
        /// Whether the target exists.
        target_exists: bool,
    },
    /// Gas cost for `DELEGATECALL`.
    DelegateCall {
        /// Call gas.
        gas: U256,
        /// True if target has not been previously accessed in this transaction
        target_is_cold: bool,
        /// Whether the target exists.
        target_exists: bool,
    },
    /// Gas cost for `STATICCALL`.
    StaticCall {
        /// Call gas.
        gas: U256,
        /// True if target has not been previously accessed in this transaction
        target_is_cold: bool,
        /// Whether the target exists.
        target_exists: bool,
    },
    /// Gas cost for `SUICIDE`.
    Suicide {
        /// Value.
        value: U256,
        /// True if target has not been previously accessed in this transaction
        target_is_cold: bool,
        /// Whether the target exists.
        target_exists: bool,
        /// Whether the target has already been removed.
        already_removed: bool,
    },
    /// Gas cost for `SSTORE`.
    SStore {
        /// Original value.
        original: H256,
        /// Current value.
        current: H256,
        /// New value.
        new: H256,
        /// True if target has not been previously accessed in this transaction
        target_is_cold: bool,
    },
    /// Gas cost for `SHA3`.
    Sha3 {
        /// Length of the data.
        len: U256,
    },
    /// Gas cost for `LOG`.
    Log {
        /// Topic length.
        n: u8,
        /// Data length.
        len: U256,
    },
    /// Gas cost for `EXTCODECOPY`.
    ExtCodeCopy {
        /// True if target has not been previously accessed in this transaction
        target_is_cold: bool,
        /// Length.
        len: U256,
    },
    /// Gas cost for some copy opcodes that is documented as `VERYLOW`.
    VeryLowCopy {
        /// Length.
        len: U256,
    },
    /// Gas cost for `EXP`.
    Exp {
        /// Power of `EXP`.
        power: U256,
    },
    /// Gas cost for `CREATE`.
    Create,
    /// Gas cost for `CREATE2`.
    Create2 {
        /// Length.
        len: U256,
    },
    /// Gas cost for `SLOAD`.
    SLoad {
        /// True if target has not been previously accessed in this transaction
        target_is_cold: bool,
    },
}

/// Storage opcode will access. Used for tracking accessed storage (EIP-2929).
#[derive(Debug, Clone, Copy)]
pub enum StorageTarget {
    /// No storage access
    None,
    /// Accessing address
    Address(H160),
    /// Accessing storage slot within an address
    Slot(H160, H256),
}

/// Memory cost.
#[derive(Debug, Clone, Copy)]
pub struct MemoryCost {
    /// Affected memory offset.
    pub offset: U256,
    /// Affected length.
    pub len: U256,
}

/// Transaction cost.
#[derive(Debug, Clone, Copy)]
pub enum TransactionCost {
    /// Call transaction cost.
    Call {
        /// Length of zeros in transaction data.
        zero_data_len: usize,
        /// Length of non-zeros in transaction data.
        non_zero_data_len: usize,
        /// Number of addresses in transaction access list (see EIP-2930)
        access_list_address_len: usize,
        /// Total number of storage keys in transaction access list (see EIP-2930)
        access_list_storage_len: usize,
    },
    /// Create transaction cost.
    Create {
        /// Length of zeros in transaction data.
        zero_data_len: usize,
        /// Length of non-zeros in transaction data.
        non_zero_data_len: usize,
        /// Number of addresses in transaction access list (see EIP-2930)
        access_list_address_len: usize,
        /// Total number of storage keys in transaction access list (see EIP-2930)
        access_list_storage_len: usize,
    },
}

impl MemoryCost {
    /// Join two memory cost together.
    pub fn join(self, other: MemoryCost) -> MemoryCost {
        if self.len == U256::zero() {
            return other;
        }

        if other.len == U256::zero() {
            return self;
        }

        let self_end = self.offset.saturating_add(self.len);
        let other_end = other.offset.saturating_add(other.len);

        if self_end >= other_end {
            self
        } else {
            other
        }
    }
}
