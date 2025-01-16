// pragma solidity ^0.8.13;
//
// contract ContractsFactory {
//     constructor() payable {}
//
//     function build()
//         public
//         returns (Item itemAddress)
//     {
//         return new Item();
//     }
//
//     // This method must be called by the contract owner. We do not check this invariant
//     // directly for the sake of simplicity of the test.
//     function withdrawAll() external {
//         address owner = msg.sender; // `msg.sender` is sender of current call = contract deployer
//         (bool success,) = owner.call{value:address(this).balance}("");
//         require(success, "Transfer failed!");
//     }
// }
//
// contract Item {}

export default {
  abi: [
    {
      type: "constructor",
      inputs: [],
      stateMutability: "payable",
    },
    {
      type: "function",
      name: "build",
      inputs: [],
      outputs: [
        {
          name: "itemAddress",
          type: "address",
          internalType: "contract Item",
        },
      ],
      stateMutability: "nonpayable",
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
    "0x60806040526101be806100115f395ff3fe608060405234801561000f575f5ffd5b5060043610610034575f3560e01c8063853828b6146100385780638e1a55fc14610042575b5f5ffd5b610040610066565b005b61004a6100f8565b6040516001600160a01b03909116815260200160405180910390f35b60405133905f90829047908381818185875af1925050503d805f81146100a7576040519150601f19603f3d011682016040523d82523d5f602084013e6100ac565b606091505b50509050806100f45760405162461bcd60e51b815260206004820152601060248201526f5472616e73666572206661696c65642160801b604482015260640160405180910390fd5b5050565b5f60405161010590610124565b604051809103905ff08015801561011e573d5f5f3e3d5ffd5b50905090565b6058806101318339019056fe6080604052348015600e575f5ffd5b50603e80601a5f395ff3fe60806040525f5ffdfea2646970667358221220e02e387d1a85a7a87458fa7991778220c10a6f75a3dac715eecf7226b473182e64736f6c634300081c0033a2646970667358221220172ab85a678765fd7f45ad56c71a71dcf4151067f255901770ad95a83feac8a464736f6c634300081c0033",
} as const;
