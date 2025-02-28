// /**
//  * @title Evm Swap Interface
//  *
//  * An interface enabling swapping the funds from EVM accounts to
//  * native Substrate accounts.
//  *
//  * Address: 0x0000000000000000000000000000000000000901
//  */
// interface EvmSwap {
//   /**
//    * Transfer the funds from an EVM account to native substrate account.
//    * Selector: 76467cbd
//    *
//    * @param nativeAddress The native address to send the funds to.
//    * @return success Whether or not the swap was successful.
//    */
//   function swap(bytes32 nativeAddress) external payable returns (bool success);
// }

export default {
  abi: [
    {
      inputs: [
        {
          internalType: "bytes32",
          name: "nativeAddress",
          type: "bytes32",
        },
      ],
      name: "swap",
      outputs: [
        {
          internalType: "bool",
          name: "success",
          type: "bool",
        },
      ],
      stateMutability: "payable",
      type: "function",
    },
    {
      anonymous: false,
      inputs: [
        {
          indexed: true,
          internalType: "address",
          name: "from",
          type: "address",
        },
        { indexed: true, internalType: "address", name: "to", type: "bytes32" },
        {
          indexed: false,
          internalType: "uint256",
          name: "value",
          type: "uint256",
        },
      ],
      name: "Swap",
      type: "event",
    },
  ],
} as const;
