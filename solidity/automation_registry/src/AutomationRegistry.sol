// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;

import {AutomationStorage} from "./AutomationStorage.sol";
import {IAutomationController} from "./IAutomationController.sol";
import {Ownable2StepUpgradeable} from "../lib/openzeppelin-contracts-upgradeable/contracts/access/Ownable2StepUpgradeable.sol";
import {PausableUpgradeable} from "../lib/openzeppelin-contracts-upgradeable/contracts/utils/PausableUpgradeable.sol";
import {UUPSUpgradeable} from "../lib/openzeppelin-contracts/contracts/proxy/utils/UUPSUpgradeable.sol";

contract AutomationRegistry is Ownable2StepUpgradeable, PausableUpgradeable, UUPSUpgradeable {
    using AutomationStorage for *;

    /// constants for task state
    uint8 constant PENDING = 0;
    uint8 constant ACTIVE = 1;
    uint8 constant CANCELLED = 2;

    /// refund fraction
    uint8 constant REFUND_FRACTION = 2;
    
    /// constants defining task type
    uint8 constant UST =  1;
    uint8 constant GST = 2;

    /// @dev Disables the initialization for implementation contract.
    constructor() {
        _disableInitializers();
    }

    /// @notice Initializes the configuration parameters of registry, can only be called once.
    function initialize(
        // supra_framework: &signer,
        uint64 _taskDurationCap,
        uint64 _registryMaxGasCap,
        uint64 _automationBaseFeeWeiPerSec,
        uint64 _flatRegistrationFeeWei,
        uint8 _congestionThresholdBps,
        uint64 _congestionBaseFeeWeiPerSec,
        uint8 _congestionExponent,
        uint16 _userTaskCapacity,
        uint64 _cycleDurationSecs,
        uint64 _sysTaskDurationCapSecs,
        uint64 _sysRegistryMaxGasCap,
        uint16 _sysTaskCapacity
    ) public initializer {
        AutomationStorage.RegistryConfig storage s = AutomationStorage.registryConfig();

        s.taskDurationCap = _taskDurationCap;
        s.registryMaxGasCap = _registryMaxGasCap;
        s.automationBaseFeeWeiPerSec = _automationBaseFeeWeiPerSec;
        s.flatRegistrationFeeWei = _flatRegistrationFeeWei;
        s.congestionThresholdBps = _congestionThresholdBps;
        s.congestionBaseFeeWeiPerSec = _congestionBaseFeeWeiPerSec;
        s.congestionExponent = _congestionExponent;
        s.userTaskCapacity = _userTaskCapacity;
        s.cycleDurationSecs = _cycleDurationSecs;
        s.sysTaskDurationCapSecs = _sysTaskDurationCapSecs;
        s.sysRegistryMaxGasCap = _sysRegistryMaxGasCap;
        s.sysTaskCapacity = _sysTaskCapacity;
        s.registrationEnabled = true;

        __Ownable2Step_init();
        __Pausable_init();
    }

    function validate_task_duration(
        uint64 _registration_time,
        uint64 _expiry_time
    ) private {

    }
    
    /// @notice Function used to register a task.
    function register(
        address _owner,
        address _target,
        bytes memory _payload,
        uint64 _expiryTime,
        bytes32 _txHash,
        uint64 _maxGasAmount,
        uint64 _gasPriceCap,
        uint64 _automationFeeCapForCycle,
        uint8[] memory _auxData
    ) private {
        // if registry is enabled
        // task priority
        // input validation
        // check cycle

        require(_owner != address(0) && _target != address(0), "Invalid address");
        require(_maxGasAmount != 0 && _gasPriceCap != 0, "Invalid gas limits");
        // require(IAutomationController.getCycleState() == 1, "Cycle not started");

        // validate_task_duration(block.timestamp, _expiryTime, _automationFeeCapForCycle);
    }


    /// @notice Helper function that reverts when 'msg.sender' is not authorized to upgrade the contract.
    /// @dev called by 'upgradeTo' and 'upgradeToAndCall' in UUPSUpgradeable
    /// @dev must be called by 'owner'
    /// @param newImplementation address of the new implementation
    function _authorizeUpgrade(address newImplementation) internal virtual override onlyOwner{ }
}
