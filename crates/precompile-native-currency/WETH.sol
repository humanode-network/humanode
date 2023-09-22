// SPDX-License-Identifier: UNLICENSED

pragma solidity >=0.7.0 <0.9.0;

/**
 * @title WETH Interface
 *
 * An interface exposing native eHMND tokens as IWETH-like contract.
 *
 * Address: 0x0000000000000000000000000000000000000802
 */
interface IWETH {
    /**
     * Provide compatibility for contracts that expect wETH design.
     * Returns funds to sender as this precompile tokens and the native tokens are the same.
     * Selector: d0e30db0
     */
    function deposit() external payable;

    /**
     * Provide compatibility for contracts that expect wETH design.
     * Does nothing.
     * Selector: 2e1a7d4d
     *
     * @param value The amount to withdraw.
     */
    function withdraw(uint256 value) external;

    /**
     * Transfer token for a specified address.
     * Selector: a9059cbb
     *
     * @param to The address to transfer tokens to.
     * @param value The amount to be transferred.
     * @return true if the transfer was succesful, revert otherwise.
     */
    function transfer(address to, uint256 value) external returns (bool);

    /**
     * Event emited when deposit has been called.
     * Selector: e1fffcc4923d04b559f4d29a8bfc6cda04eb5b0d3c460751c2402c5c5cc9109c
     *
     * @param owner Owner of the tokens.
     * @param value The amount of tokens "wrapped".
     */
    event Deposit(address indexed owner, uint256 value);

    /**
     * Event emited when withdraw has been called.
     * Selector: 7fcf532c15f0a6db0bd6d0e038bea71d30d808c7d98cb3bf7268a95bf5081b65
     *
     * @param owner Owner of the tokens.
     * @param value The amount of tokens "unwrapped".
     */
    event Withdrawal(address indexed owner, uint256 value);

    /**
     * Event emited when a transfer has been performed.
     * Selector: ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef
     *
     * @param from The address sending the tokens
     * @param to The address receiving the tokens.
     * @param value The amount of tokens transfered.
     */
    event Transfer(address indexed from, address indexed to, uint256 value);
}
