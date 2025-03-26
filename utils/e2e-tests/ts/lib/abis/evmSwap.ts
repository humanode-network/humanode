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
