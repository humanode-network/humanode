# Template Walkthrough

Benchmark produces output which uses Handlebar template to format the output.

## Usage

Provide this as input to argument `--template` in _benchmark_ command.
Default template will be used if nothing is provided (<https://github.com/humanode-network/substrate/blob/a447f03727ffd84020682f3e0e4044d5d282d274/utils/frame/benchmarking-cli/src/pallet/template.hbs>).

```bash
./humanode-peer benchmark pallet ... --template .../template.hbs
```

## How it works?

See <https://github.com/humanode-network/substrate/blob/a447f03727ffd84020682f3e0e4044d5d282d274/utils/frame/benchmarking-cli/src/pallet/writer.rs#L275-L278>.
