// pragma solidity ^0.8.13;
//
// contract ContractsFactory {
//     address private owner;
//
//     /**
//      * Sets contract deployer as owner.
//      */
//     constructor() {
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
//     function deposit() external payable {}
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
      stateMutability: "nonpayable",
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
      name: "deposit",
      inputs: [],
      outputs: [],
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
    "0x6080604052348015600e575f5ffd5b505f80546001600160a01b0319163317905561022d8061002d5f395ff3fe608060405260043610610033575f3560e01c8063853828b6146100375780638e1a55fc1461004d578063d0e30db01461004b575b5f5ffd5b348015610042575f5ffd5b5061004b61007d565b005b348015610058575f5ffd5b50610061610167565b6040516001600160a01b03909116815260200160405180910390f35b5f546001600160a01b031633146100d15760405162461bcd60e51b815260206004820152601360248201527221b0b63632b91034b9903737ba1037bbb732b960691b60448201526064015b60405180910390fd5b5f80546040516001600160a01b039091169047908381818185875af1925050503d805f811461011b576040519150601f19603f3d011682016040523d82523d5f602084013e610120565b606091505b50509050806101645760405162461bcd60e51b815260206004820152601060248201526f5472616e73666572206661696c65642160801b60448201526064016100c8565b50565b5f60405161017490610193565b604051809103905ff08015801561018d573d5f5f3e3d5ffd5b50905090565b6058806101a08339019056fe6080604052348015600e575f5ffd5b50603e80601a5f395ff3fe60806040525f5ffdfea2646970667358221220d3bdf3d2bfc6fbce1d0d2d42bf9323942c2a6cd1a0e747494d15f535449b35db64736f6c634300081c0033a26469706673582212202a42ba32ac9354cd2dbd675ca2ba09c51d4b8fdcc3ef5bd015da91685e8c113b64736f6c634300081c0033",
} as const;
