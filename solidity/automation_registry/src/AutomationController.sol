// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;

import "./AutomationStorage.sol";
import {Ownable2StepUpgradeable} from "../lib/openzeppelin-contracts-upgradeable/contracts/access/Ownable2StepUpgradeable.sol";
import {PausableUpgradeable} from "../lib/openzeppelin-contracts-upgradeable/contracts/utils/PausableUpgradeable.sol";
import {UUPSUpgradeable} from "../lib/openzeppelin-contracts/contracts/proxy/utils/UUPSUpgradeable.sol";

contract AutomationController is Ownable2StepUpgradeable, PausableUpgradeable, UUPSUpgradeable {
    using AutomationStorage for *;

    /// constants for cycle states
    uint8 constant CYCLE_READY = 0;
    uint8 constant CYCLE_STARTED = 1;
    uint8 constant CYCLE_FINISHED = 2;
    uint8 constant CYCLE_SUSPENDED = 3;
    
    /// @dev Disables the initialization for implementation contract.
    constructor() {
        _disableInitializers();
    }
    
    function initialize() public initializer {
        __Ownable2Step_init();
        __Pausable_init();
    }


    function getCycleState() public view returns(uint8) {
        return AutomationStorage.automationCycleInfo().state;
    }


    /// @notice Helper function that reverts when 'msg.sender' is not authorized to upgrade the contract.
    /// @dev called by 'upgradeTo' and 'upgradeToAndCall' in UUPSUpgradeable
    /// @dev must be called by 'owner'
    /// @param newImplementation address of the new implementation
    function _authorizeUpgrade(address newImplementation) internal virtual override onlyOwner{ }
}
