use clap::{Parser, Subcommand};
use serde_json::json;
use rand::Rng; // For generating a random part of the simulated hash

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,

    #[clap(long, default_value = "http://localhost:9944")]
    node_url: String,

    #[clap(long, default_value = "5C4hrfjw9DjXZTzV3MwzstNcxkN6odQVsreqgradKZLGHG8L")] // Dummy contract address
    contract_address: String,
}

#[derive(Subcommand, Debug)]
enum Commands {
    // Admin commands
    /// Admin: Register a new company. Caller is assumed to be contract owner.
    RegisterCompany { company_id: String },
    /// Admin: Register a new bank. Caller is assumed to be contract owner.
    RegisterBank { bank_id: String },
    /// Admin: Register a new tax office. Caller is assumed to be contract owner.
    RegisterTaxOffice { office_id: String },
    /// Admin: Set age eligibility for a pensioner. Caller is assumed to be contract owner.
    SetAgeEligibility { pensioner_id: String, is_eligible: bool },

    // Company commands
    /// Company: Update pensioner's employment details.
    UpdateEmployment {
        #[clap(long)]
        company_id_as_caller: String,
        pensioner_id: String,
        years: u32,
        salary: u128,
        status: String, // "Active", "LongTermPause", "LaidOff"
    },

    // Bank commands
    /// Bank: Add insurance details for a pensioner.
    AddInsurance {
        #[clap(long)]
        bank_id_as_caller: String,
        pensioner_id: String,
        amount: u128,
        details: String,
    },

    // Tax Office commands
    /// Tax Office: Set tax rate for a pensioner.
    SetTax {
        #[clap(long)]
        office_id_as_caller: String,
        pensioner_id: String,
        rate: u8,
    },

    // Pensioner commands
    /// Pensioner: Get estimated future payout for self.
    GetMyPayoutEstimate {
        #[clap(long)]
        pensioner_id_as_caller: String,
    },
    /// Pensioner: Initiate pension payout for self.
    InitiateMyPension {
        #[clap(long)]
        pensioner_id_as_caller: String,
    },
    /// Pensioner: Designate a spouse beneficiary for self.
    DesignateSpouse {
        #[clap(long)]
        pensioner_id_as_caller: String,
        spouse_id: String,
    },
    /// Pensioner: Get own spouse death benefit.
    GetMySpouseBenefit {
        #[clap(long)]
        spouse_id_as_caller: String, // This is the spouse checking their benefit
    },

    // General commands
    /// General: Get data for a specific pensioner.
    GetPensionerData { pensioner_id: String },
    /// General: Report the death of a pensioner.
    ReportDeath {
        #[clap(long)] // Who is reporting the death
        caller_id: String,
        deceased_pensioner_id: String,
    },
    /// General: Get the contract owner.
    GetContractOwner,
}


struct RpcClient {
    node_url: String,
    client: reqwest::Client,
}

impl RpcClient {
    pub fn new(node_url: String) -> Self {
        RpcClient {
            node_url,
            client: reqwest::Client::new(),
        }
    }

    async fn call_contract_query(
        &self,
        contract_address: &str,
        method_name: &str,
        params: serde_json::Value,
        caller_id: &str, // To simulate who is making the query
    ) -> Result<serde_json::Value, String> {
        println!(
            "Simulating QUERY from '{}' to method '{}' on contract '{}' at URL '{}'. Params: {}",
            caller_id, method_name, contract_address, self.node_url, params
        );

        match method_name {
            "get_contract_owner" => Ok(json!({
                "success": true,
                "data": { "owner": "0xAliceAliceAliceAliceAliceAliceAliceAliceAliceAliceAliceAliceAlice" } // Example owner AccountId
            })),
            "get_pensioner_data" => Ok(json!({
                "success": true,
                "data": { // Example of a more complex data structure
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
                // Simulate some logic based on caller_id if needed, or just return generic
                Ok(json!({"success": true, "data": {"estimated_payout": 12345, "currency": "Units"} }))
            }
            "get_my_spouse_death_benefit" => {
                 Ok(json!({"success": true, "data": {"benefit_amount": 5000, "currency": "Units"} }))
            }
            _ => Err(format!("Method '{}' not implemented in query simulation.", method_name)),
        }
    }

    async fn call_contract_command(
        &self,
        contract_address: &str,
        method_name: &str,
        params: serde_json::Value,
        caller_id: &str, // To simulate the origin of the transaction
    ) -> Result<serde_json::Value, String> {
        println!(
            "Simulating COMMAND from '{}' to method '{}' on contract '{}' at URL '{}'. Params: {}",
            caller_id, method_name, contract_address, self.node_url, params
        );
        
        let mut rng = rand::thread_rng();
        let random_num: u32 = rng.gen();

        // Simulate a generic successful transaction response
        Ok(json!({
            "success": true,
            "transaction_hash": format!("simulated_tx_hash_{:x}", random_num) // Use hex for typical hash appearance
        }))
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let client = RpcClient::new(cli.node_url.clone()); // Clone node_url or ensure RpcClient::new takes &str

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
