// SPDX-License-Identifier: UNLICENSED

pragma solidity >=0.7.0 <0.9.0;

/**
 * @title ERC20 Interface
 *
 * An interface exposing native eHMND tokens as ERC20 tokens.
 *
 * Address: 0x0000000000000000000000000000000000000802
 */
interface IERC20 {
    /**
     * Returns the name of the token.
     * Selector: 06fdde03
     */
    function name() external view returns (string memory);

    /**
     * Returns the symbol of the token.
     * Selector: 95d89b41
     */
    function symbol() external view returns (string memory);

    /**
     * Returns the decimals places of the token.
     * Selector: 313ce567
     */
    function decimals() external view returns (uint8);

    /**
     * Total number of tokens in existence.
     * Selector: 18160ddd
     */
    function totalSupply() external view returns (uint256);

    /**
     * Gets the balance of the specified address.
     * Selector: 70a08231
     *
     * @param owner The address to query the balance of.
     * @return uint256 The amount owned by the passed address.
     */
    function balanceOf(address owner) external view returns (uint256);

    /**
     * Function to check the amount of tokens that an owner allowed to a spender.
     * Selector: dd62ed3e
     *
     * @param owner The address which owns the funds.
     * @param spender The address which will spend the funds.
     * @return uint256 The amount of tokens still available for the spender.
     */
    function allowance(
        address owner,
        address spender
    ) external view returns (uint256);

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
     * Approve the passed address to spend the specified amount of tokens on behalf of msg.sender.
     * Selector: 095ea7b3
     *
     * @param spender The address which will spend the funds.
     * @param value The amount of tokens to be spent.
     * @return true, this cannot fail.
     */
    function approve(address spender, uint256 value) external returns (bool);

    /**
     * Transfer tokens from one address to another.
     * Selector: 23b872dd
     *
     * @param from The address which you want to send tokens from.
     * @param to The address which you want to transfer to.
     * @param value The amount of tokens to be transferred.
     * @return true if the transfer was succesful, revert otherwise.
     */
    function transferFrom(
        address from,
        address to,
        uint256 value
    ) external returns (bool);

    /**
     * Event emited when a transfer has been performed.
     * Selector: ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef
     *
     * @param from The address sending the tokens
     * @param to The address receiving the tokens.
     * @param value The amount of tokens transfered.
     */
    event Transfer(address indexed from, address indexed to, uint256 value);

    /**
     * Event emited when an approval has been registered.
     * Selector: 8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925
     *
     * @param owner The owner address of the tokens.
     * @param spender The allowed spender address.
     * @param value The amount of tokens approved.
     */
    event Approval(
        address indexed owner,
        address indexed spender,
        uint256 value
    );
}
