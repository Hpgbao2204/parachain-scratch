use parachain_template_runtime::pallet_zk_snarks::{
	common::{prepare_proof, prepare_verification_key},
	deserialization::{deserialize_public_inputs, Proof, VKey},
	verify::{prepare_public_inputs, verify},
};

use std::path::PathBuf;
use std::{fs::File, io::Read};

/// Sub-commands supported by the collator.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
    /// Build a chain specification.
    BuildSpec(sc_cli::BuildSpecCmd),

    /// Validate blocks.
    CheckBlock(sc_cli::CheckBlockCmd),

    /// Export blocks.
    ExportBlocks(sc_cli::ExportBlocksCmd),

    /// Export the state of a given block into a chain spec.
    ExportState(sc_cli::ExportStateCmd),

    /// Import blocks.
    ImportBlocks(sc_cli::ImportBlocksCmd),

    /// Revert the chain to a previous state.
    Revert(sc_cli::RevertCmd),

    /// Remove the whole chain.
    PurgeChain(cumulus_client_cli::PurgeChainCmd),

    /// Export the genesis head data of the parachain.
    ///
    /// Head data is the encoded block header.
    #[command(alias = "export-genesis-state")]
    ExportGenesisHead(cumulus_client_cli::ExportGenesisHeadCommand),

    /// Export the genesis wasm of the parachain.
    ExportGenesisWasm(cumulus_client_cli::ExportGenesisWasmCommand),

    /// Sub-commands concerned with benchmarking.
    /// The pallet benchmarking moved to the `pallet` sub-command.
    #[command(subcommand)]
    Benchmark(frame_benchmarking_cli::BenchmarkCmd),

    ZkSnarksVerify(ZkSnarksVerifyCmd),
}

const AFTER_HELP_EXAMPLE: &str = color_print::cstr!(
    r#"<bold><underline>Examples:</></>
   <bold>parachain-from-scratch-node build-spec --disable-default-bootnode > plain-parachain-chainspec.json</>
           Export a chainspec for a local testnet in json format.
   <bold>parachain-from-scratch-node --chain plain-parachain-chainspec.json --tmp -- --chain rococo-local</>
           Launch a full node with chain specification loaded from plain-parachain-chainspec.json.
   <bold>parachain-from-scratch-node</>
           Launch a full node with default parachain <italic>local-testnet</> and relay chain <italic>rococo-local</>.
   <bold>parachain-from-scratch-node --collator</>
           Launch a collator with default parachain <italic>local-testnet</> and relay chain <italic>rococo-local</>.
 "#
);

#[derive(Debug, Clone, clap::Parser)]
pub struct ZkSnarksVerifyCmd {
	#[allow(missing_docs)]
	pub vk_path: String,

	#[allow(missing_docs)]
	pub proof_path: String,

	#[allow(missing_docs)]
	pub inputs_path: String,
}

impl ZkSnarksVerifyCmd {
	pub fn run(&self) -> sc_cli::Result<()> {
		let mut vk_file = File::open(&self.vk_path)?;
		let mut vk_contents = String::new();
		vk_file.read_to_string(&mut vk_contents)?;

		let mut proof_file = File::open(&self.proof_path)?;
		let mut proof_contents = String::new();
		proof_file.read_to_string(&mut proof_contents)?;

		let mut inputs_file = File::open(&self.inputs_path)?;
		let mut inputs_contents = String::new();
		inputs_file.read_to_string(&mut inputs_contents)?;

		let vk = VKey::from_json_u8_slice(vk_contents.as_bytes()).unwrap();
		let proof = Proof::from_json_u8_slice(proof_contents.as_bytes()).unwrap();
		let inputs = deserialize_public_inputs(inputs_contents.as_bytes()).unwrap();

		match verify(
			prepare_verification_key(vk).unwrap(),
			prepare_proof(proof).unwrap(),
			prepare_public_inputs(inputs),
		) {
			Ok(true) => println!("Proof OK"),
			Ok(false) => println!("Proof NOK"),
			Err(_) => println!("Verification error"),
		}
		Ok(())
	}
}

#[derive(Debug, clap::Parser)]
#[command(
    propagate_version = true,
    args_conflicts_with_subcommands = true,
    subcommand_negates_reqs = true
)]
#[clap(after_help = AFTER_HELP_EXAMPLE)]
pub struct Cli {
    #[command(subcommand)]
    pub subcommand: Option<Subcommand>,

    #[command(flatten)]
    pub run: cumulus_client_cli::RunCmd,

    /// Disable automatic hardware benchmarks.
    ///
    /// By default these benchmarks are automatically ran at startup and measure
    /// the CPU speed, the memory bandwidth and the disk speed.
    ///
    /// The results are then printed out in the logs, and also sent as part of
    /// telemetry, if telemetry is enabled.
    #[arg(long)]
    pub no_hardware_benchmarks: bool,

    /// Relay chain arguments
    #[arg(raw = true)]
    pub relay_chain_args: Vec<String>,
}

#[derive(Debug)]
pub struct RelayChainCli {
    /// The actual relay chain cli object.
    pub base: polkadot_cli::RunCmd,

    /// Optional chain id that should be passed to the relay chain.
    pub chain_id: Option<String>,

    /// The base path that should be used by the relay chain.
    pub base_path: Option<PathBuf>,
}

impl RelayChainCli {
    /// Parse the relay chain CLI parameters using the para chain `Configuration`.
    pub fn new<'a>(
        para_config: &sc_service::Configuration,
        relay_chain_args: impl Iterator<Item = &'a String>,
    ) -> Self {
        let extension = crate::chain_spec::Extensions::try_get(&*para_config.chain_spec);
        let chain_id = extension.map(|e| e.relay_chain.clone());
        let base_path = para_config.base_path.path().join("polkadot");
        Self {
            base_path: Some(base_path),
            chain_id,
            base: clap::Parser::parse_from(relay_chain_args),
        }
    }
}
