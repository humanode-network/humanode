// pragma solidity ^0.8.0;
//
// contract SelfDestructExample {
//      address payable owner;
//
//      constructor() payable {
//         owner = payable(msg.sender);
//      }
//
//      receive() external payable {} // added for the contract to directly receive funds
//
//      function close() public {
//          require(msg.sender == owner, "Only the contract owner can call this function");
//          selfdestruct(payable (address(this)));
//      }
// }

export default {
  abi: [
    {
      inputs: [],
      stateMutability: "payable",
      type: "constructor",
    },
    {
      inputs: [],
      name: "close",
      outputs: [],
      stateMutability: "nonpayable",
      type: "function",
    },
    {
      stateMutability: "payable",
      type: "receive",
    },
  ],
  bytecode:
    "0x6080604052335f806101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055506101bc806100505f395ff3fe608060405260043610610021575f3560e01c806343d726d61461002c57610028565b3661002857005b5f80fd5b348015610037575f80fd5b50610040610042565b005b5f8054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff163373ffffffffffffffffffffffffffffffffffffffff16146100cf576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004016100c690610168565b60405180910390fd5b3073ffffffffffffffffffffffffffffffffffffffff16ff5b5f82825260208201905092915050565b7f4f6e6c792074686520636f6e7472616374206f776e65722063616e2063616c6c5f8201527f20746869732066756e6374696f6e000000000000000000000000000000000000602082015250565b5f610152602e836100e8565b915061015d826100f8565b604082019050919050565b5f6020820190508181035f83015261017f81610146565b905091905056fea264697066735822122090bbaa1686a4b74be9e2fb36847bc403059f7dca3425c045f80d157f9ef0287064736f6c63430008190033",
} as const;
