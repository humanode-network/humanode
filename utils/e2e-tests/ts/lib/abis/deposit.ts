// pragma solidity ^0.8.13;
//
// contract Deposit {
//     constructor() payable {}
//
//     function withdrawAll() external {
//         (bool success, ) = msg.sender.call{value: address(this).balance}("");
//         require(success, "Transfer failed!");
//     }
// }

export default {
  abi: [
    {
      type: "constructor",
      inputs: [],
      stateMutability: "payable",
    },
    {
      type: "function",
      name: "withdrawAll",
      inputs: [],
      outputs: [],
      stateMutability: "nonpayable",
    },
  ],
  bytecode:
    "0x608060405260f4806100105f395ff3fe6080604052348015600e575f5ffd5b50600436106026575f3560e01c8063853828b614602a575b5f5ffd5b60306032565b005b6040515f90339047908381818185875af1925050503d805f8114606f576040519150601f19603f3d011682016040523d82523d5f602084013e6074565b606091505b505090508060bb5760405162461bcd60e51b815260206004820152601060248201526f5472616e73666572206661696c65642160801b604482015260640160405180910390fd5b5056fea2646970667358221220ce7ea06cb9a3afb69b76e54771df61ff4f8e2efa0fabbb5f7ca5fb868448768664736f6c634300081c0033",
} as const;
