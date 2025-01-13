// pragma solidity ^0.8.13;
//
// contract ContractsFactory {
//     address private owner;
//
//     /**
//      * Sets contract deployer as owner.
//      */
//     constructor() payable {
//         owner = msg.sender; // 'msg.sender' is sender of current call, contract deployer for a constructor
//     }
//
//     function build()
//         public
//         returns (Item itemAddress)
//     {
//         return new Item();
//     }
//
//     function withdrawAll() external isOwner {
//         (bool success,) = owner.call{value:address(this).balance}("");
//         require(success, "Transfer failed!");
//     }
//
//     /**
//      * Makes sure if caller is owner.
//      */
//     modifier isOwner() {
//         require(msg.sender == owner, "Caller is not owner");
//         _;
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
    "0x60806040525f80546001600160a01b03191633179055610216806100225f395ff3fe608060405234801561000f575f5ffd5b5060043610610034575f3560e01c8063853828b6146100385780638e1a55fc14610042575b5f5ffd5b610040610066565b005b61004a610150565b6040516001600160a01b03909116815260200160405180910390f35b5f546001600160a01b031633146100ba5760405162461bcd60e51b815260206004820152601360248201527221b0b63632b91034b9903737ba1037bbb732b960691b60448201526064015b60405180910390fd5b5f80546040516001600160a01b039091169047908381818185875af1925050503d805f8114610104576040519150601f19603f3d011682016040523d82523d5f602084013e610109565b606091505b505090508061014d5760405162461bcd60e51b815260206004820152601060248201526f5472616e73666572206661696c65642160801b60448201526064016100b1565b50565b5f60405161015d9061017c565b604051809103905ff080158015610176573d5f5f3e3d5ffd5b50905090565b6058806101898339019056fe6080604052348015600e575f5ffd5b50603e80601a5f395ff3fe60806040525f5ffdfea26469706673582212205b805df397effd81fac6da3aaca5e17656240493717a9d8af48b1834ed60974964736f6c634300081c0033a2646970667358221220d09615e692fa242f639400bccc3a539415c571f8d26c76da4c90437f6e7ef43964736f6c634300081c0033",
} as const;
