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

  /**
   * Event emitted when a transfer has been performed.
   * Selector: 69d31d1d87c1206beee49fbab11570a7f001121cf21fbb234d5b0a2473fa5c58
   *
   * @param from The EVM account id the tokens withdrawed from.
   * @param to The Native account id the tokens deposited to.
   * @param value The amount of tokens swapped.
   */
  event Swap(address indexed from, bytes32 indexed to, uint256 value);
}
