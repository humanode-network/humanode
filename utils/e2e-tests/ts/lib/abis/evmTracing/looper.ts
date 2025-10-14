// pragma solidity >=0.8.3;
//
// contract Looper {
//     uint256 public count;
//
//     function infinite() public pure {
//         while (true) {}
//     }
//
//     function incrementalLoop(uint256 n) public {
//         uint256 i = 0;
//         while (i < n) {
//             count = count + 1;
//             i += 1;
//         }
//     }
// }

export default {
  abi: [
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
      inputs: [
        {
          internalType: "uint256",
          name: "n",
          type: "uint256",
        },
      ],
      name: "incrementalLoop",
      outputs: [],
      stateMutability: "nonpayable",
      type: "function",
    },
    {
      inputs: [],
      name: "infinite",
      outputs: [],
      stateMutability: "pure",
      type: "function",
    },
  ],
  bytecode:
    "0x6080604052348015600e575f5ffd5b506101ed8061001c5f395ff3fe608060405234801561000f575f5ffd5b506004361061003f575f3560e01c806306661abd146100435780635bec9e67146100615780636e4709f91461006b575b5f5ffd5b61004b610087565b60405161005891906100e5565b60405180910390f35b61006961008c565b005b6100856004803603810190610080919061012c565b610095565b005b5f5481565b5b600161008d57565b5f5f90505b818110156100c95760015f546100b09190610184565b5f819055506001816100c29190610184565b905061009a565b5050565b5f819050919050565b6100df816100cd565b82525050565b5f6020820190506100f85f8301846100d6565b92915050565b5f5ffd5b61010b816100cd565b8114610115575f5ffd5b50565b5f8135905061012681610102565b92915050565b5f60208284031215610141576101406100fe565b5b5f61014e84828501610118565b91505092915050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52601160045260245ffd5b5f61018e826100cd565b9150610199836100cd565b92508282019050808211156101b1576101b0610157565b5b9291505056fea26469706673582212202018b089162ffbb47bf3ea8487f4a122e87b63c21811d615bec9e32546e3f58064736f6c634300081e0033",
} as const;
