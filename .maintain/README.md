# Template Walkthrough

Benchmark produces output which uses Handlebar template to format the output.

## Usage

Provide this as input to argument `--template` in _benchmark_ command. [Default template](https://github.com/humanode-network/substrate/blob/master/utils/frame/benchmarking-cli/src/pallet/template.hbs) will be used if nothing is provided.

```bash
./humanode benchmark pallet ... --template /PATH/TO/DIRECTORY/.maintain/frame-weight-template.hbs
```

## How it works?

Under the hood, binding of variables to template takes place in [`fn write_results()`](https://github.com/humanode-network/substrate/blob/master/utils/frame/benchmarking-cli/src/pallet/writer.rs#L254). It takes the benchmark output in the form of `BenchmarkBatchSplitResults` and convert them
to `TemplateData` via [`fn map_results()`](https://github.com/humanode-network/substrate/blob/master/utils/frame/benchmarking-cli/src/pallet/writer.rs#L116). `TemplateData` is then bounded to template to produce a usable _weights.rs_ file.
[`fn write_to_result()`]() is the function that is responsible for combining template and benchmark result into usable `weights.rs`.

For brevity, `TemplateData` is as follows and only these fields are permitted in Handlebar templates:

```rust
// This is the final structure we will pass to the Handlebars template.
#[derive(Serialize, Default, Debug, Clone)]
struct TemplateData {
	args: Vec<String>,
	date: String,
	version: String,
	pallet: String,
	instance: String,
	header: String,
	cmd: CmdData,
	benchmarks: Vec<BenchmarkData>,
}

// This was the final data we have about each benchmark.
#[derive(Serialize, Default, Debug, Clone)]
struct BenchmarkData {
	name: String,
	components: Vec<Component>,
	#[serde(serialize_with = "string_serialize")]
	base_weight: u128,
	#[serde(serialize_with = "string_serialize")]
	base_reads: u128,
	#[serde(serialize_with = "string_serialize")]
	base_writes: u128,
	component_weight: Vec<ComponentSlope>,
	component_reads: Vec<ComponentSlope>,
	component_writes: Vec<ComponentSlope>,
	comments: Vec<String>,
}

// This forwards some specific metadata from the `PalletCmd`.
#[derive(Serialize, Default, Debug, Clone)]
struct CmdData {
	steps: u32,
	repeat: u32,
	lowest_range_values: Vec<u32>,
	highest_range_values: Vec<u32>,
	execution: String,
	wasm_execution: String,
	chain: String,
	db_cache: u32,
	analysis_choice: String,
}

// This encodes the component name and whether that component is used.
#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
struct Component {
	name: String,
	is_used: bool,
}

// This encodes the slope of some benchmark related to a component.
#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
struct ComponentSlope {
	name: String,
	#[serde(serialize_with = "string_serialize")]
	slope: u128,
	#[serde(serialize_with = "string_serialize")]
	error: u128,
}
```
