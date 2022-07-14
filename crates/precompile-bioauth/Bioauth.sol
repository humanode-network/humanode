// SPDX-License-Identifier: UNLICENSED

pragma solidity >=0.7.0 <0.9.0;

/**
 * @title Pallet Bioauth Interface
 *
 * The interface through which solidity contracts will interact with bioauth.
 * Address: 0x0000000000000000000000000000000000000800
 */
interface Bioauth {
    /**
     * Check if a humanode address is authenticated.
     * Selector: e3c90bb9
     *
     * @param humanode_address The humanode address to check.
     * @return Whether or not the address is authenticated.
     */
    function isAuthenticated(bytes32 humanode_address) external returns (bool);
}

