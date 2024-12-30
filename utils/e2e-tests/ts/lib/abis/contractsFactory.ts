// pragma solidity ^0.8.13;
//
// contract ContractsFactory {
//     function build()
//         public
//         returns (Item itemAddress)
//     {
//         return new Item();
//     }
// }
//
// contract Item {}

export default {
  "abi": [
    {
      "type": "function",
      "name": "build",
      "inputs": [],
      "outputs": [
        {
          "name": "itemAddress",
          "type": "address",
          "internalType": "contract Item"
        }
      ],
      "stateMutability": "nonpayable"
    }
  ],
  "bytecode": "0x6080604052348015600e575f5ffd5b5061010f8061001c5f395ff3fe6080604052348015600e575f5ffd5b50600436106026575f3560e01c80638e1a55fc14602a575b5f5ffd5b6030604c565b6040516001600160a01b03909116815260200160405180910390f35b5f6040516057906075565b604051809103905ff080158015606f573d5f5f3e3d5ffd5b50905090565b6058806100828339019056fe6080604052348015600e575f5ffd5b50603e80601a5f395ff3fe60806040525f5ffdfea2646970667358221220792811ad984520d0fcc81e7965d60d949a45e844a276b72e02a9efd3a03f105864736f6c634300081c0033a26469706673582212207f6145ff575fd357fb707898e54cf8e8193fa1187fa2649da64a06e0b092a32864736f6c634300081c0033",
} as const;
