// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;

interface IAutomationController {
    function getCycleState() external view returns(uint8);
}
