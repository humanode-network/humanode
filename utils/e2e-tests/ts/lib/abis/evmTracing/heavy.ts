// pragma solidity >=0.8.3;
//
// contract Heavy {
//     constructor(bool should_revert) {
//         if (should_revert) {
//             revert();
//         }
//     }
//
//     function call_ok() public pure {}
//
//     function call_revert() public pure {
//         revert();
//     }
//
//     function subcalls(address target0, address target1) public pure {
//         try Heavy(target0).subsubcalls(target1) {} catch {}
//         try Heavy(target0).subsubcalls(target1) {} catch {}
//     }
//
//     function subsubcalls(address target1) public pure {
//         Heavy(target1).call_ok();
//         Heavy(target1).call_revert();
//     }
//
//     function heavy_steps(uint256 store_steps, uint256 op_steps) external {
//         while (store_steps != 0) {
//             assembly {
//                 sstore(store_steps, store_steps)
//             }
//             store_steps--;
//         }
//
//         while (op_steps != 0) {
//             op_steps--;
//         }
//     }
//
//     // This part is to trace Wasm memory overflow
//     uint256 public a;
//     uint256 public b;
//     uint256 public c;
//     uint256 public d;
//     uint256 public e;
//     uint256 public f;
//     uint256 public g;
//     uint256 public h;
//     uint256 public i;
//     uint256 public j;
//
//     function set_and_loop(uint256 loops) public returns (uint256 result) {
//         a = 1;
//         b = 1;
//         c = 1;
//         d = 1;
//         e = 1;
//         f = 1;
//         g = 1;
//         h = 1;
//         i = 1;
//         j = 1;
//         uint256 count = 0;
//         while (i < loops) {
//             count += 1;
//         }
//         return 1;
//     }
// }

export default {
  abi: [
    {
      inputs: [
        {
          internalType: "bool",
          name: "should_revert",
          type: "bool",
        },
      ],
      stateMutability: "nonpayable",
      type: "constructor",
    },
    {
      inputs: [],
      name: "a",
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
      name: "b",
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
      name: "c",
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
      name: "call_ok",
      outputs: [],
      stateMutability: "pure",
      type: "function",
    },
    {
      inputs: [],
      name: "call_revert",
      outputs: [],
      stateMutability: "pure",
      type: "function",
    },
    {
      inputs: [],
      name: "d",
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
      name: "e",
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
      name: "f",
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
      name: "g",
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
      name: "h",
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
          name: "store_steps",
          type: "uint256",
        },
        {
          internalType: "uint256",
          name: "op_steps",
          type: "uint256",
        },
      ],
      name: "heavy_steps",
      outputs: [],
      stateMutability: "nonpayable",
      type: "function",
    },
    {
      inputs: [],
      name: "i",
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
      name: "j",
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
          name: "loops",
          type: "uint256",
        },
      ],
      name: "set_and_loop",
      outputs: [
        {
          internalType: "uint256",
          name: "result",
          type: "uint256",
        },
      ],
      stateMutability: "nonpayable",
      type: "function",
    },
    {
      inputs: [
        {
          internalType: "address",
          name: "target0",
          type: "address",
        },
        {
          internalType: "address",
          name: "target1",
          type: "address",
        },
      ],
      name: "subcalls",
      outputs: [],
      stateMutability: "pure",
      type: "function",
    },
    {
      inputs: [
        {
          internalType: "address",
          name: "target1",
          type: "address",
        },
      ],
      name: "subsubcalls",
      outputs: [],
      stateMutability: "pure",
      type: "function",
    },
  ],
  bytecode:
    "0x6080604052348015600e575f5ffd5b506040516108503803806108508339818101604052810190602e91906070565b80156037575f5ffd5b506096565b5f5ffd5b5f8115159050919050565b6052816040565b8114605b575f5ffd5b50565b5f81519050606a81604b565b92915050565b5f602082840312156082576081603c565b5b5f608d84828501605e565b91505092915050565b6107ad806100a35f395ff3fe608060405234801561000f575f5ffd5b50600436106100fe575f3560e01c8063b582ec5f11610095578063e2179b8e11610064578063e2179b8e14610250578063e5aa3d581461026e578063f34f16101461028c578063ffae15ba146102a8576100fe565b8063b582ec5f146101ec578063b8c9d3651461020a578063c3da42b814610228578063cb30e69614610246576100fe565b80635eaf9bc1116100d15780635eaf9bc11461018c5780636422847b146101965780638a054ac2146101b2578063a885f4e3146101d0576100fe565b80630dbe671f1461010257806313128fdc1461012057806326121ff0146101505780634df7e3d01461016e575b5f5ffd5b61010a6102c6565b6040516101179190610555565b60405180910390f35b61013a6004803603810190610135919061059c565b6102cb565b6040516101479190610555565b60405180910390f35b610158610347565b6040516101659190610555565b60405180910390f35b61017661034d565b6040516101839190610555565b60405180910390f35b610194610353565b005b6101b060048036038101906101ab91906105c7565b610355565b005b6101ba610392565b6040516101c79190610555565b60405180910390f35b6101ea60048036038101906101e5919061065f565b610398565b005b6101f461044f565b6040516102019190610555565b60405180910390f35b610212610455565b60405161021f9190610555565b60405180910390f35b61023061045b565b60405161023d9190610555565b60405180910390f35b61024e610461565b005b610258610465565b6040516102659190610555565b60405180910390f35b61027661046b565b6040516102839190610555565b60405180910390f35b6102a660048036038101906102a1919061068a565b610471565b005b6102b0610537565b6040516102bd9190610555565b60405180910390f35b5f5481565b5f60015f8190555060018081905550600160028190555060016003819055506001600481905550600160058190555060016006819055506001600781905550600160088190555060016009819055505f5f90505b82600854101561033d5760018161033691906106f5565b905061031f565b6001915050919050565b60055481565b60015481565b565b5b5f821461037357818255818061036b90610728565b925050610356565b5b5f811461038e57808061038690610728565b915050610374565b5050565b60035481565b8073ffffffffffffffffffffffffffffffffffffffff16635eaf9bc16040518163ffffffff1660e01b81526004015f6040518083038186803b1580156103dc575f5ffd5b505afa1580156103ee573d5f5f3e3d5ffd5b505050508073ffffffffffffffffffffffffffffffffffffffff1663cb30e6966040518163ffffffff1660e01b81526004015f6040518083038186803b158015610436575f5ffd5b505afa158015610448573d5f5f3e3d5ffd5b5050505050565b60095481565b60075481565b60025481565b5f5ffd5b60065481565b60085481565b8173ffffffffffffffffffffffffffffffffffffffff1663a885f4e3826040518263ffffffff1660e01b81526004016104aa919061075e565b5f6040518083038186803b1580156104c0575f5ffd5b505afa9250505080156104d1575060015b508173ffffffffffffffffffffffffffffffffffffffff1663a885f4e3826040518263ffffffff1660e01b815260040161050b919061075e565b5f6040518083038186803b158015610521575f5ffd5b505afa925050508015610532575060015b505050565b60045481565b5f819050919050565b61054f8161053d565b82525050565b5f6020820190506105685f830184610546565b92915050565b5f5ffd5b61057b8161053d565b8114610585575f5ffd5b50565b5f8135905061059681610572565b92915050565b5f602082840312156105b1576105b061056e565b5b5f6105be84828501610588565b91505092915050565b5f5f604083850312156105dd576105dc61056e565b5b5f6105ea85828601610588565b92505060206105fb85828601610588565b9150509250929050565b5f73ffffffffffffffffffffffffffffffffffffffff82169050919050565b5f61062e82610605565b9050919050565b61063e81610624565b8114610648575f5ffd5b50565b5f8135905061065981610635565b92915050565b5f602082840312156106745761067361056e565b5b5f6106818482850161064b565b91505092915050565b5f5f604083850312156106a05761069f61056e565b5b5f6106ad8582860161064b565b92505060206106be8582860161064b565b9150509250929050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52601160045260245ffd5b5f6106ff8261053d565b915061070a8361053d565b9250828201905080821115610722576107216106c8565b5b92915050565b5f6107328261053d565b91505f8203610744576107436106c8565b5b600182039050919050565b61075881610624565b82525050565b5f6020820190506107715f83018461074f565b9291505056fea26469706673582212203a48031a3a6911e44a2dfe636fa7c6f4de7214d2c0461bd2ff205366af22bb6664736f6c634300081e0033",
} as const;
