//! # Off-Chain CLI Client for Pension Management
//!
//! This Rust application provides a command-line interface (CLI) to interact with
//! the `pension_manager` smart contract. It allows users to perform various operations
//! such as registering entities, updating pensioner data, and querying contract state.
//!
//! **Note on Simulation:** Currently, this client simulates interactions and does not
//! connect to a live blockchain node or smart contract. It prints the actions it
//! would take and returns predefined or randomized responses. This is for demonstration
//! and development purposes. Future work would involve integrating actual blockchain
//! communication, including SCALE encoding/decoding of parameters and proper
//! JSON-RPC request construction for Substrate-based nodes.

use clap::{Parser, Subcommand};
use serde_json::json;
use rand::Rng; // For generating a random part of the simulated hash

/// Main CLI structure for parsing command-line arguments.
///
/// Defines global options such as the node URL and contract address,
/// and includes a subcommand for specific operations.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Specifies the subcommand to execute.
    #[clap(subcommand)]
    command: Commands,

    /// The URL of the Substrate node to connect to.
    #[clap(long, default_value = "http://localhost:9944")]
    node_url: String,

    /// The address of the deployed `pension_manager` smart contract.

    #[clap(long, default_value = "5C4hrfjw9DjXZTzV3MwzstNcxkN6odQVsreqgradKZLGHG8L")] // Dummy contract address
    contract_address: String,
}

/// Enum defining all available subcommands for the CLI.
///
/// Each variant corresponds to a specific action that can be performed on the
/// `pension_manager` smart contract. Many commands include `*_as_caller` flags
/// to simulate the identity of the transaction sender for authorization purposes
/// in the current simulation mode.
#[derive(Subcommand, Debug)]
enum Commands {
    // Admin commands
    /// Admin: Register a new company.
    /// In a real scenario, this would be callable only by the contract owner.
    /// The `caller_id` for the simulation is assumed to be the contract owner.
    RegisterCompany { 
        /// The AccountId (as a string) of the company to register.
        company_id: String 
    },
    /// Admin: Register a new bank.
    /// The `caller_id` for the simulation is assumed to be the contract owner.
    RegisterBank { 
        /// The AccountId (as a string) of the bank to register.
        bank_id: String 
    },
    /// Admin: Register a new tax office.
    /// The `caller_id` for the simulation is assumed to be the contract owner.
    RegisterTaxOffice { 
        /// The AccountId (as a string) of the tax office to register.
        office_id: String 
    },
    /// Admin: Set age eligibility for a pensioner.
    /// The `caller_id` for the simulation is assumed to be the contract owner.
    SetAgeEligibility { 
        /// The AccountId (as a string) of the pensioner.
        pensioner_id: String, 
        /// Boolean flag indicating if the pensioner is age-eligible.
        is_eligible: bool 
    },

    // Company commands
    /// Company: Update pensioner's employment details.
    /// The `--company-id-as-caller` flag simulates the company making this call.
    UpdateEmployment {
        /// The AccountId (as a string) of the authorized company making the call.
        #[clap(long)]
        company_id_as_caller: String,
        /// The AccountId (as a string) of the pensioner to update.
        pensioner_id: String,
        /// New total years worked for the pensioner.
        years: u32,
        /// New current salary for the pensioner.
        salary: u128,
        /// New employment status (e.g., "Active", "LongTermPause", "LaidOff").
        status: String,
    },

    // Bank commands
    /// Bank: Add insurance details for a pensioner.
    /// The `--bank-id-as-caller` flag simulates the bank making this call.
    AddInsurance {
        /// The AccountId (as a string) of the authorized bank making the call.
        #[clap(long)]
        bank_id_as_caller: String,
        /// The AccountId (as a string) of the pensioner.
        pensioner_id: String,
        /// Insurance payout amount per period.
        amount: u128,
        /// Details of the insurance policy.
        details: String,
    },

    // Tax Office commands
    /// Tax Office: Set tax rate for a pensioner.
    /// The `--office-id-as-caller` flag simulates the tax office making this call.
    SetTax {
        /// The AccountId (as a string) of the authorized tax office making the call.
        #[clap(long)]
        office_id_as_caller: String,
        /// The AccountId (as a string) of the pensioner.
        pensioner_id: String,
        /// Tax rate percentage (0-100).

        rate: u8,
    },

    // Pensioner commands
    /// Pensioner: Get estimated future payout for self.
    /// The `--pensioner-id-as-caller` flag simulates the pensioner making this query.
    GetMyPayoutEstimate {
        /// The AccountId (as a string) of the pensioner making the query.
        #[clap(long)]
        pensioner_id_as_caller: String,
    },
    /// Pensioner: Initiate pension payout for self.
    /// The `--pensioner-id-as-caller` flag simulates the pensioner making this call.
    InitiateMyPension {
        /// The AccountId (as a string) of the pensioner making the call.
        #[clap(long)]
        pensioner_id_as_caller: String,
    },
    /// Pensioner: Designate a spouse beneficiary for self.
    /// The `--pensioner-id-as-caller` flag simulates the pensioner making this call.
    DesignateSpouse {
        /// The AccountId (as a string) of the pensioner making the call.
        #[clap(long)]
        pensioner_id_as_caller: String,
        /// The AccountId (as a string) of the spouse to be designated.
        spouse_id: String,
    },
    /// Pensioner (Spouse): Get own spouse death benefit.
    /// The `--spouse-id-as-caller` flag simulates the spouse checking their benefit.
    GetMySpouseBenefit {
        /// The AccountId (as a string) of the spouse beneficiary making the query.
        #[clap(long)]
        spouse_id_as_caller: String,
    },

    // General commands
    /// General: Get data for a specific pensioner.
    /// This is a public query; caller identity is less critical in simulation.
    GetPensionerData { 
        /// The AccountId (as a string) of the pensioner whose data is requested.
        pensioner_id: String 
    },
    /// General: Report the death of a pensioner.
    /// This action can be performed by any caller in the simulation.
    ReportDeath {
        /// The AccountId (as a string) of the entity reporting the death.
        #[clap(long)] 
        caller_id: String,
        /// The AccountId (as a string) of the pensioner who is deceased.
        deceased_pensioner_id: String,
    },
    /// General: Get the contract owner.
    /// This is a public query.
    GetContractOwner,
}

/// `RpcClient` is responsible for simulating interactions with the smart contract.
///
/// It holds a `reqwest::Client` for potential future HTTP requests and the target node's URL.
/// Currently, its methods simulate contract calls rather than performing actual network operations.
struct RpcClient {
    /// URL of the Substrate node where the contract is (conceptually) deployed.
    node_url: String,
    /// HTTP client for making requests. Not fully utilized in simulation mode.
    client: reqwest::Client,
}

impl RpcClient {
    /// Creates a new `RpcClient`.
    ///
    /// # Arguments
    /// * `node_url`: The URL of the target Substrate node.
    pub fn new(node_url: String) -> Self {
        RpcClient {
            node_url,
            client: reqwest::Client::new(),
        }
    }

    /// Simulates a read-only query to the smart contract.
    ///
    /// This function logs the intended query details and returns a predefined
    /// JSON response based on the `method_name`. It does not perform actual
    /// network calls or SCALE encoding/decoding.
    ///
    /// # Arguments
    /// * `contract_address`: The address of the target smart contract (string).
    /// * `method_name`: The name of the contract message to call (e.g., "get_contract_owner").
    /// * `params`: JSON-formatted parameters for the contract call.
    /// * `caller_id`: The AccountId (as a string) of the entity simulating the query.
    ///
    /// # Returns
    /// A `Result` containing a `serde_json::Value` on successful simulation,
    /// or a `String` error message if the method is not recognized in the simulation.
    async fn call_contract_query(
        &self,
        contract_address: &str,
        method_name: &str,
        params: serde_json::Value,
        caller_id: &str, 
    ) -> Result<serde_json::Value, String> {
        println!(
            "Simulating QUERY from '{}' to method '{}' on contract '{}' at URL '{}'. Params: {}",
            caller_id, method_name, contract_address, self.node_url, params
        );

        match method_name {
            "get_contract_owner" => Ok(json!({
                "success": true,
                "data": { "owner": "0xAliceAliceAliceAliceAliceAliceAliceAliceAliceAliceAliceAliceAlice" }
            })),
            "get_pensioner_data" => Ok(json!({
                "success": true,
                "data": { 
                    "years_worked": params.get("years_worked").unwrap_or(&json!(10)).as_u64().unwrap_or(10),
                    "current_salary": params.get("current_salary").unwrap_or(&json!(50000)).as_u64().unwrap_or(50000),
                    "status": "Active",
                    "is_deceased": false,
                    "is_receiving_pension": false,
                    "is_eligible_for_payout_age_wise": false,
                    "pension_payout_amount": null,
                    "spouse_beneficiary": null
                }
            })),
            "get_my_future_payout" => {
                Ok(json!({"success": true, "data": {"estimated_payout": 12345, "currency": "Units"} }))
            }
            "get_my_spouse_death_benefit" => {
                 Ok(json!({"success": true, "data": {"benefit_amount": 5000, "currency": "Units"} }))
            }
            _ => Err(format!("Method '{}' not implemented in query simulation.", method_name)),
        }
    }

    /// Simulates a transactional command (state-changing call) to the smart contract.
    ///
    /// This function logs the intended command details and returns a simulated
    /// successful transaction response, including a randomized transaction hash.
    /// It does not perform actual network calls or SCALE encoding/decoding.
    ///
    /// # Arguments
    /// * `contract_address`: The address of the target smart contract (string).
    /// * `method_name`: The name of the contract message to call (e.g., "register_company").
    /// * `params`: JSON-formatted parameters for the contract call.
    /// * `caller_id`: The AccountId (as a string) of the entity simulating the transaction.
    ///
    /// # Returns
    /// A `Result` containing a `serde_json::Value` representing the simulated
    /// transaction success (with a fake hash), or a `String` error message.
    async fn call_contract_command(
        &self,
        contract_address: &str,
        method_name: &str,
        params: serde_json::Value,
        caller_id: &str, 
    ) -> Result<serde_json::Value, String> {
        println!(
            "Simulating COMMAND from '{}' to method '{}' on contract '{}' at URL '{}'. Params: {}",
            caller_id, method_name, contract_address, self.node_url, params
        );
        
        let mut rng = rand::thread_rng();
        let random_num: u32 = rng.gen();

        Ok(json!({
            "success": true,
            "transaction_hash": format!("simulated_tx_hash_{:x}", random_num)
        }))
    }
}

/// Main entry point for the off-chain client application.
///
/// Parses command-line arguments, instantiates the `RpcClient`, and dispatches
/// the appropriate simulated contract call based on the provided subcommand.
/// Prints the result of the simulated operation to the console.
#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let client = RpcClient::new(cli.node_url.clone());

    let contract_address = &cli.contract_address;
    let result = match cli.command {
        Commands::RegisterCompany { company_id } => {
            let params = json!({ "company_id": company_id });
            client.call_contract_command(contract_address, "register_company", params, "assumed_contract_owner").await
        }
        Commands::RegisterBank { bank_id } => {
            let params = json!({ "bank_id": bank_id });
            client.call_contract_command(contract_address, "register_bank", params, "assumed_contract_owner").await
        }
        Commands::RegisterTaxOffice { office_id } => {
            let params = json!({ "tax_office_id": office_id });
            client.call_contract_command(contract_address, "register_tax_office", params, "assumed_contract_owner").await
        }
        Commands::SetAgeEligibility { pensioner_id, is_eligible } => {
            let params = json!({ "pensioner_id": pensioner_id, "is_eligible": is_eligible });
            client.call_contract_command(contract_address, "set_age_eligibility_status", params, "assumed_contract_owner").await
        }
        Commands::UpdateEmployment { company_id_as_caller, pensioner_id, years, salary, status } => {
            let params = json!({
                "pensioner_id": pensioner_id,
                "years_worked": years,
                "current_salary": salary,
                "status": status // Passed as string, e.g., "Active"
            });
            client.call_contract_command(contract_address, "update_pensioner_employment", params, &company_id_as_caller).await
        }
        Commands::AddInsurance { bank_id_as_caller, pensioner_id, amount, details } => {
            let params = json!({
                "pensioner_id": pensioner_id,
                "insurance_payout_per_period": amount,
                "details": details
            });
            client.call_contract_command(contract_address, "add_pension_insurance", params, &bank_id_as_caller).await
        }
        Commands::SetTax { office_id_as_caller, pensioner_id, rate } => {
            let params = json!({
                "pensioner_id": pensioner_id,
                "tax_rate_percentage": rate
            });
            client.call_contract_command(contract_address, "apply_pension_tax_rate", params, &office_id_as_caller).await
        }
        Commands::GetMyPayoutEstimate { pensioner_id_as_caller } => {
            // This contract call `get_my_future_payout` takes no parameters in the contract itself.
            client.call_contract_query(contract_address, "get_my_future_payout", json!({}), &pensioner_id_as_caller).await
        }
        Commands::InitiateMyPension { pensioner_id_as_caller } => {
            // This contract call `initiate_pension_payout` takes no parameters in the contract itself.
            client.call_contract_command(contract_address, "initiate_pension_payout", json!({}), &pensioner_id_as_caller).await
        }
        Commands::DesignateSpouse { pensioner_id_as_caller, spouse_id } => {
            let params = json!({ "spouse_id": spouse_id });
            client.call_contract_command(contract_address, "designate_spouse_beneficiary", params, &pensioner_id_as_caller).await
        }
        Commands::GetMySpouseBenefit { spouse_id_as_caller } => {
            // This contract call `get_my_spouse_death_benefit` takes no parameters in the contract itself.
            client.call_contract_query(contract_address, "get_my_spouse_death_benefit", json!({}), &spouse_id_as_caller).await
        }
        Commands::GetPensionerData { pensioner_id } => {
            let params = json!({ "pensioner_id": pensioner_id });
            // For a general query, the "caller" might be a generic default or not strictly relevant if data is public
            client.call_contract_query(contract_address, "get_pensioner_data", params, "any_caller_for_query").await
        }
        Commands::ReportDeath { caller_id, deceased_pensioner_id } => {
            let params = json!({ "deceased_pensioner_id": deceased_pensioner_id });
            client.call_contract_command(contract_address, "report_death_and_assign_spouse_benefit", params, &caller_id).await
        }
        Commands::GetContractOwner => {
            client.call_contract_query(contract_address, "get_contract_owner", json!({}), "any_caller_for_query").await
        }
    };

    match result {
        Ok(value) => println!("Operation successful. Response:\n{}", serde_json::to_string_pretty(&value).unwrap_or_else(|e| format!("Error pretty printing JSON: {}",e))),
        Err(e) => eprintln!("Operation failed: {}", e),
    }
}
