# EVM integration

## Account

We currently have only a single notion of an account stored on chain.
There is no such notion as "Substrate *account*" or "EVM *account*" -
the accounts space is uniform and is not divided into Substrate or EVM.
We will call it Humanode account.

Account on the chain are essentially a key-value map, where keys are of the type
`<Runtime as frame_system::Config>::AccountId`, which is technically
an `AccountId32`.
See the code documentation on the `AccountId` type alias for more info.

## Addresses

Despite that there is only a single account type in the system, there
are multiple kinds of *addresses* to which an account can correspond.

An account always has a native address associated with it.

