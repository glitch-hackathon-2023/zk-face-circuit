use circuit::*;
use circuit::{evm_prove, gen_evm_verifier, gen_keys, gen_params, prove, verify};
use clap::{Parser, Subcommand};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand, Clone)]
enum Commands {
    /// Generate a setup parameter (not for production).
    GenParams {
        /// k parameter (cargo run gen-params --k 20)
        #[arg(long)]
        k: u32,
        /// setup parameter path
        /// #[arg(short, long, default_value = "./build/params")]
        #[arg(short, long, default_value = "./circuit/params/app.bin")]
        params_path: String,
    },
    /// Generate a proving key and a verifying key. (cargo run gen-keys)
    GenKeys {
        /// setup parameter file
        #[arg(short, long, default_value = "./circuit/params")]
        params_dir: String,
        /// circuit configure file
        #[arg(
            short = 'b',
            long,
            default_value = "./circuit/configs/test1_circuit.config"
        )]
        app_circuit_config: String,
        #[arg(
            short,
            long,
            default_value = "./circuit/configs/agg_circuit.config"
        )]
        agg_circuit_config: String,
        /// proving key file path
        #[arg(long, default_value = "./circuit/contracts/pks")]
        pk_dir: String,
        /// verifying key file path
        #[arg(long, default_value = "./circuit/contracts/app.vk")]
        vk_path: String,
    },
    Prove {
        /// setup parameter file
        #[arg(short, long, default_value = "./circuit/params")]
        params_dir: String,
        /// circuit configure file
        #[arg(
            short = 'b',
            long,
            default_value = "./circuit/configs/test1_circuit.config"
        )]
        app_circuit_config: String,
        #[arg(
            short,
            long,
            default_value = "./circuit/configs/agg_circuit.config"
        )]
        agg_circuit_config: String,
        /// proving key file path
        #[arg(long, default_value = "./circuit/contracts/pks")]
        pk_dir: String,
        /// input file path
        #[arg(long, default_value = "./circuit/contracts/input.json")]
        input_path: String,
        /// proof file path
        #[arg(long, default_value = "./circuit/contracts/proof.bin")]
        proof_path: String,
        /// public input file path
        #[arg(long, default_value = "./circuit/contracts/public_input.json")]
        public_input_path: String,
    },
    EvmProve {
        /// setup parameter file
        #[arg(short, long, default_value = "./circuit/params")]
        params_dir: String,
        /// circuit configure file
        #[arg(
            short = 'b',
            long,
            default_value = "./circuit/configs/test1_circuit.config"
        )]
        app_circuit_config: String,
        #[arg(
            short,
            long,
            default_value = "./circuit/configs/agg_circuit.config"
        )]
        agg_circuit_config: String,
        /// proving key file path
        #[arg(long, default_value = "./circuit/contracts/pks")]
        pk_dir: String,
        /// input file path
        #[arg(long, default_value = "./circuit/contracts/input.json")]
        input_path: String,
        /// proof file path
        #[arg(long, default_value = "./circuit/contracts/evm_proof.hex")]
        proof_path: String,
        /// public input file path
        #[arg(long, default_value = "./circuit/contracts/evm_public_input.json")]
        public_input_path: String,
    },
    Verify {
        /// setup parameter file
        #[arg(short, long, default_value = "./circuit/params")]
        params_dir: String,
        /// circuit configure file
        #[arg(
            short = 'b',
            long,
            default_value = "./circuit/configs/test1_circuit.config"
        )]
        app_circuit_config: String,
        #[arg(
            short,
            long,
            default_value = "./circuit/configs/agg_circuit.config"
        )]
        agg_circuit_config: String,
        /// verifying key file path
        #[arg(long, default_value = "./circuit/contracts/app.vk")]
        vk_path: String,
        /// public input file path
        #[arg(long, default_value = "./circuit/contracts/public_input.json")]
        public_input_path: String,
        /// proof file path
        #[arg(long, default_value = "./circuit/contracts/proof.bin")]
        proof_path: String,
    },
    GenEvmVerifier { // cargo run gen-evm-verifier
        /// setup parameter file
        #[arg(short, long, default_value = "./circuit/params")]
        params_dir: String,
        /// circuit configure file
        #[arg(
            short = 'b',
            long,
            default_value = "./circuit/configs/test1_circuit.config"
        )]
        app_circuit_config: String,
        #[arg(
            short,
            long,
            default_value = "./circuit/configs/agg_circuit.config"
        )]
        agg_circuit_config: String,
        /// verifying key file path
        #[arg(long, default_value = "./circuit/contracts/app.vk")]
        vk_path: String,
        /// verifier code path
        #[arg(long, default_value = "./circuit/contracts/Verifier.sol")]
        code_path: String,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::GenParams { k, params_path } => gen_params(&params_path, k).unwrap(),
        Commands::GenKeys {
            params_dir,
            app_circuit_config,
            agg_circuit_config,
            pk_dir,
            vk_path,
        } => gen_keys(
            &params_dir,
            &app_circuit_config,
            &agg_circuit_config,
            &pk_dir,
            &vk_path,
        )
        .unwrap(),
        Commands::Prove {
            params_dir,
            app_circuit_config,
            agg_circuit_config,
            pk_dir,
            input_path,
            proof_path,
            public_input_path,
        } => prove(
            &params_dir,
            &app_circuit_config,
            &agg_circuit_config,
            &pk_dir,
            &input_path,
            &proof_path,
            &public_input_path,
        )
        .unwrap(),
        Commands::EvmProve {
            params_dir,
            app_circuit_config,
            agg_circuit_config,
            pk_dir,
            input_path,
            proof_path,
            public_input_path,
        } => evm_prove(
            &params_dir,
            &app_circuit_config,
            &agg_circuit_config,
            &pk_dir,
            &input_path,
            &proof_path,
            &public_input_path,
        )
        .unwrap(),
        Commands::Verify {
            params_dir,
            app_circuit_config,
            agg_circuit_config,
            vk_path,
            public_input_path,
            proof_path,
        } => verify(
            &params_dir,
            &app_circuit_config,
            &agg_circuit_config,
            &vk_path,
            &public_input_path,
            &proof_path,
        )
        .unwrap(),
        Commands::GenEvmVerifier {
            params_dir,
            app_circuit_config,
            agg_circuit_config,
            vk_path,
            code_path,
        } => gen_evm_verifier(
            &params_dir,
            &app_circuit_config,
            &agg_circuit_config,
            &vk_path,
            &code_path,
        )
        .unwrap(),
    }
}
