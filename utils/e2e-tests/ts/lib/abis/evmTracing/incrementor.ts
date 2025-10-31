// pragma solidity >=0.8.3;
//
// contract Incrementor {
//     uint256 public count;
//
//     constructor() {
//         count = 0;
//     }
//
//     function incr() public {
//         count = count + 1;
//     }
//
//     function incr(uint256 num) public returns (uint256) {
//         count = count + num;
//         return count;
//     }
// }

export default {
  abi: [
    {
      inputs: [],
      stateMutability: "nonpayable",
      type: "constructor",
    },
    {
      inputs: [],
      name: "count",
      outputs: [
        {
          internalType: "uint256",
          name: "",
          type: "uint256",
        },
      ],
      stateMutability: "view",
      type: "function",
    },
    {
      inputs: [],
      name: "incr",
      outputs: [],
      stateMutability: "nonpayable",
      type: "function",
    },
    {
      inputs: [
        {
          internalType: "uint256",
          name: "num",
          type: "uint256",
        },
      ],
      name: "incr",
      outputs: [
        {
          internalType: "uint256",
          name: "",
          type: "uint256",
        },
      ],
      stateMutability: "nonpayable",
      type: "function",
    },
  ],
  bytecode:
    "0x6080604052348015600e575f5ffd5b505f5f819055506101f1806100225f395ff3fe608060405234801561000f575f5ffd5b506004361061003f575f3560e01c806306661abd14610043578063119fbbd41461006157806321b13c481461006b575b5f5ffd5b61004b61009b565b60405161005891906100e9565b60405180910390f35b6100696100a0565b005b61008560048036038101906100809190610130565b6100b5565b60405161009291906100e9565b60405180910390f35b5f5481565b60015f546100ae9190610188565b5f81905550565b5f815f546100c39190610188565b5f819055505f549050919050565b5f819050919050565b6100e3816100d1565b82525050565b5f6020820190506100fc5f8301846100da565b92915050565b5f5ffd5b61010f816100d1565b8114610119575f5ffd5b50565b5f8135905061012a81610106565b92915050565b5f6020828403121561014557610144610102565b5b5f6101528482850161011c565b91505092915050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52601160045260245ffd5b5f610192826100d1565b915061019d836100d1565b92508282019050808211156101b5576101b461015b565b5b9291505056fea2646970667358221220836c30d0514664def3debcdd06557178b40a286f683281477d734ff6edbfa46064736f6c634300081e0033",
} as const;
