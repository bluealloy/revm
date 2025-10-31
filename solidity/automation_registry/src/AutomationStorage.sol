
// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;

import "../lib/openzeppelin-contracts/contracts/utils/structs/EnumerableSet.sol";

// Helper library to access the storage slots. Used by AutomationRegistry and AutomationExecuter.
library AutomationStorage {
    using EnumerableSet for EnumerableSet.UintSet;

    // Storage slot used to store configuration of the registry
    bytes32 internal constant REGISTRY_CONFIG = keccak256("supra.automation.registry.config");
    
    // Storage slot used to store the registry state
    bytes32 internal constant REGISTRY_STATE = keccak256("supra.automation.registry.state");
    
    // Storage slot used to store deposits and fee related accounting
    bytes32 internal constant DEPOSIT = keccak256("supra.automation.registry.deposit");
    
    // Storage slot used to store the state of system tasks registry
    bytes32 internal constant REGISTRY_SYSTEM_TASKS = keccak256("supra.automation.registry.systemTasks");
    
    // Storage slot used to store the cycle information
    bytes32 internal constant CYCLE_INFO = keccak256("supra.automation.registry.cycleInfo");
    
    // Storage slot used to store the cycle transition information
    bytes32 internal constant TRANSITION_STATE = keccak256("supra.automation.registry.transitionState");

    /// @notice Configuration parameters for the automation registry.
    struct RegistryConfig {
        uint64 taskDurationCap;
        uint64 registryMaxGasCap;
        uint64 automationBaseFeeWeiPerSec;  // TO_DO: need to decide on the currency
        uint64 flatRegistrationFeeWei;      // TO_DO: need to decide on the currency
        uint8 congestionThresholdBps;
        uint64 congestionBaseFeeWeiPerSec;
        uint8 congestionExponent;
        uint16 userTaskCapacity;
        uint64 cycleDurationSecs;
        uint64 sysTaskDurationCapSecs;
        uint64 sysRegistryMaxGasCap;
        uint16 sysTaskCapacity;
        bool registrationEnabled;
    }

    /// @notice Tracks per-cycle automation state and task indexes for user tasks.
    struct RegistryState {
        uint64 currentIndex;
        uint64 gasCommittedForNextCycle;
        uint64 gasCommittedForThisCycle;
        EnumerableSet.UintSet activeTaskIds;
        mapping(uint64 => TaskMetadata) tasks;   
        // TO_DO: mapping(address => uint64[])  
    }

    /// @notice Tracks per-cycle automation state and task indexes for system tasks.
    struct RegistryStateSystemTasks {
        uint64 gasCommittedForNextCycle;
        uint64 gasCommittedForThisCycle;
        EnumerableSet.UintSet taskIds;
        mapping(address => bool) authorizedAccounts; // TO_DO: use enumerable set for array
    }

    /// @notice Task metadata for individual automation tasks.
    struct TaskMetadata {
        uint64 taskIndex;
        address owner;
        address target;
        bytes payload;          // TO_DO: use struct to combine target address and task payload
        uint64 expiryTime;
        bytes32 txHash;
        uint64 maxGasAmount;
        uint64 gasPriceCap;
        uint64 automationFeeCapForCycle;
        uint64 registrationTime;
        uint8 state;            // TO_DO: use enum
        uint64 lockedFeeForNextCycle;
    }

    /// @notice Deposit and fee related accounting.
    struct Deposit {
        address coldWallet;
        uint256 totalCollectedFees;
        uint256 totalLockedFees;
        mapping(address => uint256) userBalances;   // TO_DO: redundency
        mapping(uint64 => uint256) taskLockedFees;
    }

    /// @notice Struct representing the state of current cycle.
    struct AutomationCycleInfo{
        uint64 index;
        uint8 state;   // TO_DO: use enum
        uint64 startTime;
        uint64 durationSecs;
    }

    /// @notice Struct representing state transition information.
    struct TransitionState {
        uint64 refundDuration;
        uint64 newCycleDuration;
        uint64 automationFeePerSec;
        uint64 gasCommittedForNewCycle;
        uint64 gasCommittedForNextCycle;
        uint64 sysGasCommittedForNextCycle;
        uint64 lockedFees;
        EnumerableSet.UintSet expectedTasksToBeProcessed;
        uint64 nextTaskIndexPosition;
    }

    /// @notice Function to return storage reference for registry config.
    function registryConfig() internal pure returns (RegistryConfig storage s) {
        bytes32 slot = REGISTRY_CONFIG;
        assembly {
            s.slot := slot
        }
    }

    /// @notice Function to return storage reference for registry state.
    function registryState() internal pure returns (RegistryState storage s) {
        bytes32 slot = REGISTRY_STATE;
        assembly {
            s.slot := slot
        }
    }

    /// @notice Function to return storage reference for deposits.
    function deposit() internal pure returns (Deposit storage s) {
        bytes32 slot = DEPOSIT;
        assembly {
            s.slot := slot
        }
    }

    /// @notice Function to return storage reference for registry state of system tasks.
    function registryStateSystemTasks() internal pure returns (RegistryStateSystemTasks storage s) {
        bytes32 slot = REGISTRY_SYSTEM_TASKS;
        assembly {
            s.slot := slot
        }
    }

    /// @notice Function to return storage reference for cycle information.
    function automationCycleInfo() internal pure returns (AutomationCycleInfo storage s) {
        bytes32 slot = CYCLE_INFO;
        assembly {
            s.slot := slot
        }
    }

    /// @notice Function to return storage reference for transition state.
    function transitionState() internal pure returns (TransitionState storage s) {
        bytes32 slot = TRANSITION_STATE;
        assembly {
            s.slot := slot
        }
    }
}
