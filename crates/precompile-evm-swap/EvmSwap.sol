// SPDX-License-Identifier: UNLICENSED

pragma solidity >=0.7.0 <0.9.0;

/**
 * @title Evm Swap Interface
 *
 * An interface enabling swapping the funds from EVM accounts to
 * native Substrate accounts.
 *
 * Address: 0x0000000000000000000000000000000000000900
 */
interface EvmSwap {
  /**
   * Transfer the funds from an EVM account to native substrate account.
   * Selector: 76467cbd
   *
   * @param nativeAddress The native address to send the funds to.
   * @return success Whether or not the swap was successful.
   */
  function swap(bytes32 nativeAddress) external payable returns (bool success);
}
