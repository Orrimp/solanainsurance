#![cfg_attr(not(feature = "std"), no_std, no_main)]

/// The `pension_manager` ink! smart contract.
///
/// This contract manages pension schemes, allowing for the registration of pensioners,
/// companies, banks, and tax offices. It handles employment updates, insurance additions,
/// tax configurations, pension payout estimations, payout initiation, and death benefit processing.
///
/// Key functionalities include:
/// - Role-based access control for different operations (contract owner, authorized companies, banks, tax offices).
/// - Secure storage of pensioner data, insurance details, and tax configurations.
/// - Calculation of pension amounts based on employment history, salary, and additional insurances, adjusted for taxes.
/// - Management of pension payout lifecycle, including eligibility checks and death benefit distribution.
#[ink::contract]
pub mod pension_manager {
    use ink::prelude::vec::Vec;
    use ink::prelude::string::String;
    use ink::storage::Mapping;
    // Explicitly import AccountId and Balance if not covered by prelude or for clarity
    use ink::env::AccountId;
    use ink::env::Balance;

    /// Custom error types for the `PensionManager` contract.
    /// These errors are returned by callable messages to indicate failure conditions.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Caller is not authorized to perform the action.
        Unauthorized,
        /// The entity (e.g., company, bank) is already registered.
        AlreadyRegistered,
        /// The entity (e.g., company, bank) is not registered.
        NotRegistered,
        /// The specified pensioner was not found in the records.
        PensionerNotFound,
        /// An input parameter was invalid (e.g., tax rate > 100).
        InvalidInput,
        /// The requested payout operation is not applicable (e.g., pensioner is deceased or already receiving).
        PayoutNotApplicable,
        /// The pensioner is not yet eligible for payout (e.g., by age or other criteria).
        NotYetEligibleForPayout,
        /// Attempted to report death for a pensioner who is already marked as deceased.
        AlreadyDeceased,
    }

    /// Defines the employment status of a pensioner.
    #[derive(Debug, PartialEq, Eq, Clone, Copy, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum EmploymentStatus {
        /// Pensioner is actively employed.
        Active,
        /// Pensioner is on a long-term pause from employment.
        LongTermPause,
        /// Pensioner has been laid off.
        LaidOff,
    }

    /// Holds detailed information about a pensioner.
    /// This struct is stored in the `pensioners` mapping.
    #[derive(Debug, PartialEq, Eq, Clone, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct PensionerData {
        /// Total number of years the pensioner has worked.
        pub years_worked: u32,
        /// Current or last known salary of the pensioner.
        pub current_salary: Balance,
        /// Current employment status of the pensioner.
        pub status: EmploymentStatus,
        /// Flag indicating if the pensioner is deceased.
        pub is_deceased: bool,
        /// Flag indicating if the pensioner is currently receiving pension payouts.
        pub is_receiving_pension: bool,
        /// Flag indicating if the pensioner meets age-based (or other) criteria for payout eligibility.
        pub is_eligible_for_payout_age_wise: bool, 
        /// The calculated and approved pension payout amount per period, if initiated.
        pub pension_payout_amount: Option<Balance>, 
        /// Optional `AccountId` of a designated spouse beneficiary for death benefits.
        pub spouse_beneficiary: Option<AccountId>,
    }

    /// Holds information about a bank or insurance provider for a specific pensioner.
    /// This includes details about additional insurance payouts.
    #[derive(Debug, PartialEq, Eq, Clone, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct BankInsuranceInfo {
        /// `AccountId` of the bank or insurance provider.
        pub bank_id: AccountId,
        /// Additional payout amount per period from this insurance.
        pub insurance_payout_per_period: Balance,
        /// Descriptive details about the insurance policy.
        pub details: String,
    }

    /// Holds tax configuration information for a specific pensioner, applied by a tax office.
    #[derive(Debug, PartialEq, Eq, Clone, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct TaxOfficeInfo {
        /// `AccountId` of the tax office that applied this configuration.
        pub tax_office_id: AccountId,
        /// Tax rate percentage (0-100) to be applied to the pensioner's payout.
        pub tax_rate_percentage: u8,
    }

    /// Main storage struct for the `PensionManager` contract.
    /// Contains all persistent data of the pension system.
    #[ink(storage)]
    pub struct PensionManager {
        /// Mapping from a pensioner's `AccountId` to their `PensionerData`.
        pub pensioners: Mapping<AccountId, PensionerData>,
        /// Mapping to store authorized company `AccountId`s. Presence indicates authorization.
        pub company_authorizations: Mapping<AccountId, ()>,
        /// Mapping to store authorized bank `AccountId`s.
        pub bank_authorizations: Mapping<AccountId, ()>,
        /// Mapping to store authorized tax office `AccountId`s.
        pub tax_office_authorizations: Mapping<AccountId, ()>,
        /// Mapping from a pensioner's `AccountId` to a list of their `BankInsuranceInfo`.
        pub pensioner_insurances: Mapping<AccountId, Vec<BankInsuranceInfo>>,
        /// Mapping from a pensioner's `AccountId` to their `TaxOfficeInfo`.
        pub pensioner_tax_config: Mapping<AccountId, TaxOfficeInfo>,
        /// Mapping from a spouse beneficiary's `AccountId` to their calculated death benefit amount.
        pub spouse_death_benefits: Mapping<AccountId, Balance>,
        /// The `AccountId` of the contract owner, set at deployment.
        pub contract_owner: AccountId,
    }

    impl PensionManager {
        /// Constructor: Initializes a new instance of the `PensionManager` contract.
        ///
        /// Sets the caller of this constructor as the `contract_owner`.
        /// Initializes all storage mappings to be empty.
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                pensioners: Mapping::new(),
                company_authorizations: Mapping::new(),
                bank_authorizations: Mapping::new(),
                tax_office_authorizations: Mapping::new(),
                pensioner_insurances: Mapping::new(),
                pensioner_tax_config: Mapping::new(),
                spouse_death_benefits: Mapping::new(), // Initialize new mapping
                contract_owner: Self::env().caller(),
            }
        }

        // --- Private Helper for Payout Calculation ---
        /// Internal helper to calculate the gross pension amount before tax, including base and insurances.
        /// This is not directly callable but used by `get_my_future_payout` and `initiate_pension_payout`.
        fn _calculate_pension_amount(&self, pensioner_data: &PensionerData, pensioner_id: &AccountId) -> Result<Balance, Error> {
            if pensioner_data.is_deceased { // Safeguard, should ideally be checked by calling logic
                return Err(Error::PayoutNotApplicable);
            }

            let base_pension = pensioner_data.current_salary
                .checked_div(100)
                .unwrap_or(0)
                .checked_mul(Balance::from(pensioner_data.years_worked))
                .unwrap_or(0)
                .checked_mul(2)
                .unwrap_or(0);
            
            let mut total_estimated_payout = base_pension;

            if let Some(insurances) = self.pensioner_insurances.get(pensioner_id) {
                for insurance in insurances {
                    total_estimated_payout = total_estimated_payout.saturating_add(insurance.insurance_payout_per_period);
                }
            }

            let final_estimated_payout;
            if let Some(tax_info) = self.pensioner_tax_config.get(pensioner_id) {
                if tax_info.tax_rate_percentage > 100 {
                     return Err(Error::InvalidInput);
                }
                let tax_amount = total_estimated_payout
                    .checked_mul(Balance::from(tax_info.tax_rate_percentage))
                    .unwrap_or(0)
                    .checked_div(100)
                    .unwrap_or(0);
                final_estimated_payout = total_estimated_payout.saturating_sub(tax_amount);
            } else {
                final_estimated_payout = total_estimated_payout;
            }
            Ok(final_estimated_payout)
        }


        /// Ensures that the caller is an authorized bank.
        fn ensure_caller_is_authorized_bank(&self) -> Result<(), Error> {
            let caller = self.env().caller();
            if !self.bank_authorizations.contains(&caller) {
                Err(Error::Unauthorized)
            } else {
                Ok(())
            }
        }

        /// Ensures that the caller is an authorized tax office.
        fn ensure_caller_is_authorized_tax_office(&self) -> Result<(), Error> {
            let caller = self.env().caller();
            if !self.tax_office_authorizations.contains(&caller) {
                Err(Error::Unauthorized)
            } else {
                Ok(())
            }
        }

        // --- Helper Functions ---
        /// Ensures that the caller is the contract owner.
        fn ensure_owner(&self) -> Result<(), Error> {
            if self.env().caller() != self.contract_owner {
                Err(Error::Unauthorized)
            } else {
                Ok(())
            }
        }

        /// Ensures that the provided company_id is authorized.
        fn ensure_company_authorized(&self, company_id: &AccountId) -> Result<(), Error> {
            if !self.company_authorizations.contains(company_id) {
                Err(Error::Unauthorized)
            } else {
                Ok(())
            }
        }

        // --- Registration / Unregistration Messages (Owner Only) ---
        /// Registers a new company.
        ///
        /// Only the `contract_owner` can call this message.
        ///
        /// # Arguments
        /// * `company_id`: The `AccountId` of the company to register.
        ///
        /// # Errors
        /// * `Error::Unauthorized` if the caller is not the contract owner.
        /// * `Error::AlreadyRegistered` if the company is already registered.
        #[ink(message)]
        pub fn register_company(&mut self, company_id: AccountId) -> Result<(), Error> {
            self.ensure_owner()?;
            if self.company_authorizations.contains(&company_id) {
                return Err(Error::AlreadyRegistered);
            }
            self.company_authorizations.insert(company_id, &());
            Ok(())
        }

        /// Unregisters an existing company.
        ///
        /// Only the `contract_owner` can call this message.
        ///
        /// # Arguments
        /// * `company_id`: The `AccountId` of the company to unregister.
        ///
        /// # Errors
        /// * `Error::Unauthorized` if the caller is not the contract owner.
        /// * `Error::NotRegistered` if the company is not currently registered.
        #[ink(message)]
        pub fn unregister_company(&mut self, company_id: AccountId) -> Result<(), Error> {
            self.ensure_owner()?;
            if !self.company_authorizations.contains(&company_id) {
                return Err(Error::NotRegistered);
            }
            self.company_authorizations.remove(&company_id);
            Ok(())
        }

        /// Registers a new bank.
        ///
        /// Only the `contract_owner` can call this message.
        ///
        /// # Arguments
        /// * `bank_id`: The `AccountId` of the bank to register.
        ///
        /// # Errors
        /// * `Error::Unauthorized` if the caller is not the contract owner.
        /// * `Error::AlreadyRegistered` if the bank is already registered.
        #[ink(message)]
        pub fn register_bank(&mut self, bank_id: AccountId) -> Result<(), Error> {
            self.ensure_owner()?;
            if self.bank_authorizations.contains(&bank_id) {
                return Err(Error::AlreadyRegistered);
            }
            self.bank_authorizations.insert(bank_id, &());
            Ok(())
        }

        /// Unregisters an existing bank.
        ///
        /// Only the `contract_owner` can call this message.
        ///
        /// # Arguments
        /// * `bank_id`: The `AccountId` of the bank to unregister.
        ///
        /// # Errors
        /// * `Error::Unauthorized` if the caller is not the contract owner.
        /// * `Error::NotRegistered` if the bank is not currently registered.

        #[ink(message)]
        pub fn unregister_bank(&mut self, bank_id: AccountId) -> Result<(), Error> {
            self.ensure_owner()?;
            if !self.bank_authorizations.contains(&bank_id) {
                return Err(Error::NotRegistered);
            }
            self.bank_authorizations.remove(&bank_id);
            Ok(())
        }

        /// Registers a new tax office.
        ///
        /// Only the `contract_owner` can call this message.
        ///
        /// # Arguments
        /// * `tax_office_id`: The `AccountId` of the tax office to register.
        ///
        /// # Errors
        /// * `Error::Unauthorized` if the caller is not the contract owner.
        /// * `Error::AlreadyRegistered` if the tax office is already registered.
        #[ink(message)]
        pub fn register_tax_office(&mut self, tax_office_id: AccountId) -> Result<(), Error> {
            self.ensure_owner()?;
            if self.tax_office_authorizations.contains(&tax_office_id) {
                return Err(Error::AlreadyRegistered);
            }
            self.tax_office_authorizations.insert(tax_office_id, &());
            Ok(())
        }

        /// Unregisters an existing tax office.
        ///
        /// Only the `contract_owner` can call this message.
        ///
        /// # Arguments
        /// * `tax_office_id`: The `AccountId` of the tax office to unregister.
        ///
        /// # Errors
        /// * `Error::Unauthorized` if the caller is not the contract owner.
        /// * `Error::NotRegistered` if the tax office is not currently registered.

        #[ink(message)]
        pub fn unregister_tax_office(&mut self, tax_office_id: AccountId) -> Result<(), Error> {
            self.ensure_owner()?;
            if !self.tax_office_authorizations.contains(&tax_office_id) {
                return Err(Error::NotRegistered);
            }
            self.tax_office_authorizations.remove(&tax_office_id);
            Ok(())
        }

        // --- Pensioner Data Update Message (Registered Companies Only) ---

        /// Updates the employment details for a given pensioner.
        ///
        /// Only an authorized company can call this message.
        /// If the `pensioner_id` does not exist, a new record is created with default values
        /// for `is_deceased`, `is_receiving_pension`, `is_eligible_for_payout_age_wise`,
        /// `pension_payout_amount`, and `spouse_beneficiary`.
        ///
        /// # Arguments
        /// * `pensioner_id`: The `AccountId` of the pensioner to update.
        /// * `years_worked`: The new total years worked.
        /// * `current_salary`: The new current salary.
        /// * `status`: The new `EmploymentStatus`.
        ///
        /// # Errors
        /// * `Error::Unauthorized` if the caller is not an authorized company.
              
        #[ink(message)]
        pub fn update_pensioner_employment(
            &mut self,
            pensioner_id: AccountId,
            years_worked: u32,
            current_salary: Balance,
            status: EmploymentStatus,
        ) -> Result<(), Error> {
            let caller = self.env().caller();
            self.ensure_company_authorized(&caller)?; // Check if the caller is an authorized company

            let mut pensioner_data = self.pensioners.get(&pensioner_id).unwrap_or_else(|| {
                PensionerData {
                    years_worked: 0, // Will be updated
                    current_salary: 0, // Will be updated
                    status: EmploymentStatus::Active, // Default, will be updated
                    is_deceased: false,
                    is_receiving_pension: false,
                    is_eligible_for_payout_age_wise: false, // New field default
                    pension_payout_amount: None,          // New field default
                    spouse_beneficiary: None,             // New field default
                }
            });

            pensioner_data.years_worked = years_worked;
            pensioner_data.current_salary = current_salary;
            pensioner_data.status = status;
            // is_deceased and is_receiving_pension are not modified here by company

            self.pensioners.insert(pensioner_id, &pensioner_data);
            Ok(())
        }

        // --- Bank and Tax Office Messages ---

        /// Adds a pension insurance record for a specified pensioner.
        ///
        /// Only an authorized bank can call this message.
        /// The insurance details are added to the pensioner's list of insurances.
        ///
        /// # Arguments
        /// * `pensioner_id`: The `AccountId` of the pensioner.
        /// * `insurance_payout_per_period`: The payout amount per period for this insurance.
        /// * `details`: A string describing the insurance policy.
        ///
        /// # Errors
        /// * `Error::Unauthorized` if the caller is not an authorized bank.
        /// * `Error::PensionerNotFound` if the `pensioner_id` does not exist.
        /// Applies or updates the pension tax rate for a specified pensioner.
        ///
        /// Only an authorized tax office can call this message.
        ///
        /// # Arguments
        /// * `pensioner_id`: The `AccountId` of the pensioner.
        /// * `tax_rate_percentage`: The tax rate (0-100) to apply.
        ///
        /// # Errors
        /// * `Error::Unauthorized` if the caller is not an authorized tax office.
        /// * `Error::PensionerNotFound` if the `pensioner_id` does not exist.
        /// * `Error::InvalidInput` if `tax_rate_percentage` is greater than 100.
        #[ink(message)]
        pub fn add_pension_insurance(
            &mut self,
            pensioner_id: AccountId,
            insurance_payout_per_period: Balance,
            details: String, // ink::prelude::string::String
        ) -> Result<(), Error> {
            self.ensure_caller_is_authorized_bank()?;

            if !self.pensioners.contains(&pensioner_id) {
                return Err(Error::PensionerNotFound);
            }

            let insurance_info = BankInsuranceInfo {
                bank_id: self.env().caller(),
                insurance_payout_per_period,
                details,
            };

            let mut insurances = self.pensioner_insurances.get(&pensioner_id).unwrap_or_default();
            insurances.push(insurance_info);
            self.pensioner_insurances.insert(pensioner_id, &insurances);

            Ok(())
        }

        #[ink(message)]
        pub fn apply_pension_tax_rate(
            &mut self,
            pensioner_id: AccountId,
            tax_rate_percentage: u8,
        ) -> Result<(), Error> {
            self.ensure_caller_is_authorized_tax_office()?;

            if !self.pensioners.contains(&pensioner_id) {
                return Err(Error::PensionerNotFound);
            }

            if tax_rate_percentage > 100 {
                return Err(Error::InvalidInput);
            }

            let tax_info = TaxOfficeInfo {
                tax_office_id: self.env().caller(),
                tax_rate_percentage,
            };
            self.pensioner_tax_config.insert(pensioner_id, &tax_info);

            Ok(())
        }

        // --- Pensioner-Callable Messages ---

        /// Sets the age-based eligibility status for a pensioner.
        ///
        /// Only the `contract_owner` can call this message.
        /// This is a simplified mechanism for age verification; a real system might use oracles.
        ///
        /// # Arguments
        /// * `pensioner_id`: The `AccountId` of the pensioner.
        /// * `is_eligible`: Boolean flag indicating if the pensioner is age-eligible.
        ///
        /// # Errors
        /// * `Error::Unauthorized` if the caller is not the contract owner.
        /// * `Error::PensionerNotFound` if the `pensioner_id` does not exist.
        #[ink(message)]
        pub fn set_age_eligibility_status(&mut self, pensioner_id: AccountId, is_eligible: bool) -> Result<(), Error> {
            self.ensure_owner()?;
            let mut pensioner_data = self.pensioners.get_mut(&pensioner_id).ok_or(Error::PensionerNotFound)?;
            pensioner_data.is_eligible_for_payout_age_wise = is_eligible;
            self.pensioners.insert(pensioner_id, &pensioner_data);
            Ok(())
        }

        /// Allows a pensioner (the caller) to initiate their pension payout.
        ///
        /// The pensioner must exist, not be deceased, not already be receiving pension,
        /// and be marked as `is_eligible_for_payout_age_wise`.
        /// The calculated pension amount is stored, and `is_receiving_pension` is set to true.
        ///
        /// # Returns
        /// The calculated `Balance` of the pension payout per period on success.
        ///
        /// # Errors
        /// * `Error::PensionerNotFound` if the caller is not a registered pensioner.
        /// * `Error::PayoutNotApplicable` if the pensioner is deceased or already receiving pension.
        /// * `Error::NotYetEligibleForPayout` if `is_eligible_for_payout_age_wise` is false.
        /// * `Error::InvalidInput` if there's an issue with stored tax data (e.g., rate > 100).
        #[ink(message)]
        pub fn initiate_pension_payout(&mut self) -> Result<Balance, Error> {
            let caller = self.env().caller();
            let mut pensioner_data = self.pensioners.get_mut(&caller).ok_or(Error::PensionerNotFound)?;

            if pensioner_data.is_deceased || pensioner_data.is_receiving_pension {
                return Err(Error::PayoutNotApplicable);
            }
            if !pensioner_data.is_eligible_for_payout_age_wise {
                return Err(Error::NotYetEligibleForPayout);
            }

            let calculated_payout = self._calculate_pension_amount(&pensioner_data, &caller)?;
            
            pensioner_data.pension_payout_amount = Some(calculated_payout);
            pensioner_data.is_receiving_pension = true;
            self.pensioners.insert(caller, &pensioner_data);

            Ok(calculated_payout)
        }

        /// Allows a pensioner (the caller) to designate a spouse as a beneficiary.
        ///
        /// The pensioner must exist and not be deceased.
        ///
        /// # Arguments
        /// * `spouse_id`: The `AccountId` of the spouse to be designated.
        ///
        /// # Errors
        /// * `Error::PensionerNotFound` if the caller is not a registered pensioner.
        /// * `Error::PayoutNotApplicable` if the pensioner is deceased.

        #[ink(message)]
        pub fn designate_spouse_beneficiary(&mut self, spouse_id: AccountId) -> Result<(), Error> {
            let caller = self.env().caller();
            let mut pensioner_data = self.pensioners.get_mut(&caller).ok_or(Error::PensionerNotFound)?;

            if pensioner_data.is_deceased {
                return Err(Error::PayoutNotApplicable);
            }
            
            pensioner_data.spouse_beneficiary = Some(spouse_id);
            self.pensioners.insert(caller, &pensioner_data);
            Ok(())
        }
        
        /// Reports the death of a pensioner and assigns death benefits if a spouse is designated.
        ///
        /// This message can be called by anyone.
        /// It marks the pensioner as deceased, stops any ongoing pension, and if a spouse beneficiary
        /// is set, calculates a 20% death benefit based on the pensioner's last calculated payout potential
        /// and stores it for the spouse.
        ///
        /// # Arguments
        /// * `deceased_pensioner_id`: The `AccountId` of the pensioner who has deceased.
        ///
        /// # Returns
        /// `Ok(Some(Balance))` with the calculated spouse benefit if a spouse was designated,
        /// `Ok(None)` if no spouse was designated, or an `Error`.
        ///
        /// # Errors
        /// * `Error::PensionerNotFound` if `deceased_pensioner_id` does not exist.
        /// * `Error::AlreadyDeceased` if the pensioner is already marked as deceased.
        /// * `Error::InvalidInput` if there's an issue with stored tax data during benefit calculation.
        #[ink(message)]
        pub fn report_death_and_assign_spouse_benefit(&mut self, deceased_pensioner_id: AccountId) -> Result<Option<Balance>, Error> {
            let mut pensioner_data = self.pensioners.get_mut(&deceased_pensioner_id).ok_or(Error::PensionerNotFound)?;

            if pensioner_data.is_deceased {
                return Err(Error::AlreadyDeceased); 
            }

            // Calculate benefit before marking as deceased for payout calculation logic
            let benefit_base_amount = self._calculate_pension_amount(&pensioner_data, &deceased_pensioner_id)?;

            pensioner_data.is_deceased = true;
            pensioner_data.is_receiving_pension = false; // Stop pension if it was active

            let mut assigned_spouse_benefit: Option<Balance> = None;
            if let Some(spouse_id) = pensioner_data.spouse_beneficiary {
                let spouse_benefit = benefit_base_amount
                    .checked_mul(20)
                    .unwrap_or(0)
                    .checked_div(100)
                    .unwrap_or(0);
                self.spouse_death_benefits.insert(spouse_id, &spouse_benefit);
                assigned_spouse_benefit = Some(spouse_benefit);
            }
            
            self.pensioners.insert(deceased_pensioner_id, &pensioner_data);
            Ok(assigned_spouse_benefit)
        }

        /// Retrieves the estimated future pension payout for the caller (pensioner).
        ///
        /// This is a read-only query. The calculation includes base pension, added insurances,
        /// and applied taxes.
        ///
        /// # Returns
        /// The estimated `Balance` of the future payout per period on success.
        ///
        /// # Errors
        /// * `Error::PensionerNotFound` if the caller is not a registered pensioner.
        /// * `Error::PayoutNotApplicable` if the pensioner is deceased.
        /// * `Error::InvalidInput` if there's an issue with stored tax data (e.g., rate > 100).
        #[ink(message)]
        pub fn get_my_future_payout(&self) -> Result<Balance, Error> {
            let caller = self.env().caller();
            let pensioner_data = self.pensioners.get(&caller).ok_or(Error::PensionerNotFound)?;
             if pensioner_data.is_deceased {
                return Err(Error::PayoutNotApplicable);
            }
            self._calculate_pension_amount(&pensioner_data, &caller)
        }


        // --- Getter/Check Messages (Callable by Anyone) ---
              
        /// Checks if a given `AccountId` is an authorized company.
        #[ink(message)]
        pub fn is_company_authorized(&self, company_id: AccountId) -> bool {
            self.company_authorizations.contains(&company_id)
        }

        /// Checks if a given `AccountId` is an authorized bank.

        #[ink(message)]
        pub fn is_bank_authorized(&self, bank_id: AccountId) -> bool {
            self.bank_authorizations.contains(&bank_id)
        }

        /// Checks if a given `AccountId` is an authorized tax office.

        #[ink(message)]
        pub fn is_tax_office_authorized(&self, tax_office_id: AccountId) -> bool {
            self.tax_office_authorizations.contains(&tax_office_id)
        }

        /// Retrieves the `PensionerData` for a given `pensioner_id`.
        /// Returns `None` if the pensioner is not found.

        #[ink(message)]
        pub fn get_pensioner_data(&self, pensioner_id: AccountId) -> Option<PensionerData> {
            self.pensioners.get(&pensioner_id)
        }

        /// Retrieves the list of `BankInsuranceInfo` for a given `pensioner_id`.
        /// Returns `None` if the pensioner has no insurance records or is not found.

        #[ink(message)]
        pub fn get_pensioner_insurances(&self, pensioner_id: AccountId) -> Option<Vec<BankInsuranceInfo>> {
            self.pensioner_insurances.get(&pensioner_id)
        }

        /// Retrieves the `TaxOfficeInfo` for a given `pensioner_id`.
        /// Returns `None` if no tax configuration is set for the pensioner or if not found.

        #[ink(message)]
        pub fn get_pensioner_tax_config(&self, pensioner_id: AccountId) -> Option<TaxOfficeInfo> {
            self.pensioner_tax_config.get(&pensioner_id)
        }
        
        /// Retrieves the death benefit amount assigned to the caller (spouse beneficiary).
        /// Returns `None` if the caller has no death benefit assigned.

        #[ink(message)]
        pub fn get_my_spouse_death_benefit(&self) -> Option<Balance> {
            self.spouse_death_benefits.get(&self.env().caller())
        }

        /// Retrieves the `AccountId` of the contract owner.

        #[ink(message)]
        pub fn get_contract_owner(&self) -> AccountId {
            self.contract_owner
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::{test, DefaultEnvironment};

        fn default_accounts() -> test::DefaultAccounts<DefaultEnvironment> {
            test::default_accounts::<DefaultEnvironment>()
        }

        fn set_caller(caller: AccountId) {
            test::set_caller::<DefaultEnvironment>(caller);
        }

        fn get_caller() -> AccountId {
            test::callee::<DefaultEnvironment>()
        }


        #[ink::test]
        fn new_works() {
            let accounts = default_accounts();
            set_caller(accounts.alice);
            let contract = PensionManager::new();
            assert_eq!(contract.contract_owner, accounts.alice);
            assert_eq!(contract.get_contract_owner(), accounts.alice);
            let non_existent_pensioner: Option<PensionerData> = contract.pensioners.get(&accounts.bob);
            assert_eq!(non_existent_pensioner, None);
        }

        #[ink::test]
        fn registration_works() {
            let accounts = default_accounts();
            set_caller(accounts.alice);
            let mut contract = PensionManager::new();

            // Register company
            assert_eq!(contract.register_company(accounts.django), Ok(()));
            assert!(contract.is_company_authorized(accounts.django));
            assert_eq!(contract.register_company(accounts.django), Err(Error::AlreadyRegistered));

            // Unregister company
            assert_eq!(contract.unregister_company(accounts.django), Ok(()));
            assert!(!contract.is_company_authorized(accounts.django));
            assert_eq!(contract.unregister_company(accounts.django), Err(Error::NotRegistered));

            // Similar tests for bank
            assert_eq!(contract.register_bank(accounts.eve), Ok(()));
            assert!(contract.is_bank_authorized(accounts.eve));
            assert_eq!(contract.unregister_bank(accounts.eve), Ok(()));
            assert!(!contract.is_bank_authorized(accounts.eve));

            // Similar tests for tax office
            assert_eq!(contract.register_tax_office(accounts.frank), Ok(()));
            assert!(contract.is_tax_office_authorized(accounts.frank));
            assert_eq!(contract.unregister_tax_office(accounts.frank), Ok(()));
            assert!(!contract.is_tax_office_authorized(accounts.frank));
        }

        #[ink::test]
        fn registration_unauthorized() {
            let accounts = default_accounts();
            set_caller(accounts.alice);
            let mut contract = PensionManager::new();

            set_caller(accounts.bob); // Bob is not the owner
            assert_eq!(contract.register_company(accounts.django), Err(Error::Unauthorized));
            assert_eq!(contract.unregister_company(accounts.django), Err(Error::Unauthorized)); // Django not registered yet, but owner check first
            assert_eq!(contract.register_bank(accounts.eve), Err(Error::Unauthorized));
            assert_eq!(contract.unregister_bank(accounts.eve), Err(Error::Unauthorized));
            assert_eq!(contract.register_tax_office(accounts.frank), Err(Error::Unauthorized));
            assert_eq!(contract.unregister_tax_office(accounts.frank), Err(Error::Unauthorized));
        }
        
        #[ink::test]
        fn update_pensioner_employment_works() {
            let accounts = default_accounts();
            set_caller(accounts.alice); // Alice is the owner
            let mut contract = PensionManager::new();

            // Register a company
            assert_eq!(contract.register_company(accounts.django), Ok(()));

            // Django (company) updates pensioner Bob's data
            set_caller(accounts.django);
            let years = 10;
            let salary = 50000;
            let status = EmploymentStatus::Active;
            assert_eq!(contract.update_pensioner_employment(accounts.bob, years, salary, status), Ok(()));

            // Verify data
            let pensioner_data = contract.get_pensioner_data(accounts.bob).expect("Pensioner should exist");
            assert_eq!(pensioner_data.years_worked, years);
            assert_eq!(pensioner_data.current_salary, salary);
            assert_eq!(pensioner_data.status, status);
            assert!(!pensioner_data.is_deceased);
            assert!(!pensioner_data.is_receiving_pension);

            // Update existing pensioner
            let new_years = 12;
            let new_salary = 55000;
            let new_status = EmploymentStatus::LongTermPause;
            assert_eq!(contract.update_pensioner_employment(accounts.bob, new_years, new_salary, new_status), Ok(()));
            let updated_pensioner_data = contract.get_pensioner_data(accounts.bob).expect("Pensioner should exist");
            assert_eq!(updated_pensioner_data.years_worked, new_years);
            assert_eq!(updated_pensioner_data.current_salary, new_salary);
            assert_eq!(updated_pensioner_data.status, new_status);
        }

        #[ink::test]
        fn update_pensioner_unauthorized_company() {
            let accounts = default_accounts();
            set_caller(accounts.alice); // Alice is the owner
            let mut contract = PensionManager::new();

            // Charlie is NOT a registered company
            set_caller(accounts.charlie);
            assert_eq!(
                contract.update_pensioner_employment(accounts.bob, 5, 30000, EmploymentStatus::Active),
                Err(Error::Unauthorized)
            );
        }

        #[ink::test]
        fn ensure_owner_works() {
            let accounts = default_accounts();
            set_caller(accounts.alice);
            let contract = PensionManager::new();
            assert_eq!(contract.ensure_owner(), Ok(()));

            set_caller(accounts.bob);
            assert_eq!(contract.ensure_owner(), Err(Error::Unauthorized));
        }

        #[ink::test]
        fn ensure_company_authorized_works() {
            let accounts = default_accounts();
            set_caller(accounts.alice);
            let mut contract = PensionManager::new();
            
            assert_eq!(contract.register_company(accounts.django), Ok(()));
            
            // Django is authorized
            set_caller(accounts.django); // Set caller to Django for the check if needed, though ensure_company_authorized takes company_id as param
            assert_eq!(contract.ensure_company_authorized(&accounts.django), Ok(()));

            // Charlie is not authorized
            assert_eq!(contract.ensure_company_authorized(&accounts.charlie), Err(Error::Unauthorized));
        }

        #[ink::test]
        fn add_pension_insurance_works() {
            let accounts = default_accounts();
            set_caller(accounts.alice); // Owner
            let mut contract = PensionManager::new();

            // Register company and bank
            assert_eq!(contract.register_company(accounts.django), Ok(()));
            assert_eq!(contract.register_bank(accounts.eve), Ok(()));

            // Company Django registers pensioner Bob
            set_caller(accounts.django);
            assert_eq!(contract.update_pensioner_employment(accounts.bob, 5, 30000, EmploymentStatus::Active), Ok(()));

            // Bank Eve adds insurance for Bob
            set_caller(accounts.eve);
            let insurance_payout = 1000;
            let details = String::from("Life Time Basic");
            assert_eq!(contract.add_pension_insurance(accounts.bob, insurance_payout, details.clone()), Ok(()));

            // Verify insurance
            let bob_insurances = contract.get_pensioner_insurances(accounts.bob).expect("Bob should have insurances");
            assert_eq!(bob_insurances.len(), 1);
            assert_eq!(bob_insurances[0].bank_id, accounts.eve);
            assert_eq!(bob_insurances[0].insurance_payout_per_period, insurance_payout);
            assert_eq!(bob_insurances[0].details, details);

            // Add another insurance
            let insurance_payout_2 = 500;
            let details_2 = String::from("Extra Health");
            assert_eq!(contract.add_pension_insurance(accounts.bob, insurance_payout_2, details_2.clone()), Ok(()));
            let bob_insurances_updated = contract.get_pensioner_insurances(accounts.bob).expect("Bob should have insurances");
            assert_eq!(bob_insurances_updated.len(), 2);
        }

        #[ink::test]
        fn add_pension_insurance_unauthorized_bank() {
            let accounts = default_accounts();
            set_caller(accounts.alice);
            let mut contract = PensionManager::new();
            assert_eq!(contract.register_company(accounts.django), Ok(()));
            set_caller(accounts.django);
            assert_eq!(contract.update_pensioner_employment(accounts.bob, 5, 30000, EmploymentStatus::Active), Ok(()));

            set_caller(accounts.charlie); // Charlie is not a registered bank
            assert_eq!(
                contract.add_pension_insurance(accounts.bob, 1000, String::from("Test fail")),
                Err(Error::Unauthorized)
            );
        }

        #[ink::test]
        fn add_pension_insurance_pensioner_not_found() {
            let accounts = default_accounts();
            set_caller(accounts.alice);
            let mut contract = PensionManager::new();
            assert_eq!(contract.register_bank(accounts.eve), Ok(()));

            set_caller(accounts.eve);
            assert_eq!(
                contract.add_pension_insurance(accounts.bob, 1000, String::from("Test fail")),
                Err(Error::PensionerNotFound)
            );
        }

        #[ink::test]
        fn apply_pension_tax_rate_works() {
            let accounts = default_accounts();
            set_caller(accounts.alice); // Owner
            let mut contract = PensionManager::new();

            assert_eq!(contract.register_company(accounts.django), Ok(()));
            assert_eq!(contract.register_tax_office(accounts.frank), Ok(()));

            set_caller(accounts.django);
            assert_eq!(contract.update_pensioner_employment(accounts.bob, 5, 30000, EmploymentStatus::Active), Ok(()));

            set_caller(accounts.frank); // Frank is tax office
            let tax_rate = 15; // 15%
            assert_eq!(contract.apply_pension_tax_rate(accounts.bob, tax_rate), Ok(()));

            let bob_tax_config = contract.get_pensioner_tax_config(accounts.bob).expect("Bob should have tax config");
            assert_eq!(bob_tax_config.tax_office_id, accounts.frank);
            assert_eq!(bob_tax_config.tax_rate_percentage, tax_rate);
        }

        #[ink::test]
        fn apply_pension_tax_rate_invalid_rate() {
            let accounts = default_accounts();
            set_caller(accounts.alice);
            let mut contract = PensionManager::new();
            assert_eq!(contract.register_company(accounts.django), Ok(()));
            assert_eq!(contract.register_tax_office(accounts.frank), Ok(()));
            set_caller(accounts.django);
            assert_eq!(contract.update_pensioner_employment(accounts.bob, 5, 30000, EmploymentStatus::Active), Ok(()));

            set_caller(accounts.frank);
            assert_eq!(contract.apply_pension_tax_rate(accounts.bob, 101), Err(Error::InvalidInput));
        }
        
        #[ink::test]
        fn get_my_future_payout_works() {
            let accounts = default_accounts();
            set_caller(accounts.alice); // Owner
            let mut contract = PensionManager::new();

            // Register company, bank, tax office
            assert_eq!(contract.register_company(accounts.django), Ok(()));
            assert_eq!(contract.register_bank(accounts.eve), Ok(()));
            assert_eq!(contract.register_tax_office(accounts.frank), Ok(()));

            // Pensioner Bob's data by company Django
            set_caller(accounts.django);
            let salary_bob: Balance = 60000; // Bob's salary
            let years_bob: u32 = 20;       // Bob's years worked
            assert_eq!(contract.update_pensioner_employment(accounts.bob, years_bob, salary_bob, EmploymentStatus::Active), Ok(()));

            // Bank Eve adds insurance for Bob
            set_caller(accounts.eve);
            let insurance_payout_bob = 10000;
            assert_eq!(contract.add_pension_insurance(accounts.bob, insurance_payout_bob, String::from("Bob's Insurance")), Ok(()));

            // Tax office Frank sets tax rate for Bob
            set_caller(accounts.frank);
            let tax_rate_bob = 10; // 10%
            assert_eq!(contract.apply_pension_tax_rate(accounts.bob, tax_rate_bob), Ok(()));

            // Bob checks his future payout
            set_caller(accounts.bob);
            let payout_result = contract.get_my_future_payout();
            assert!(payout_result.is_ok(), "Payout calculation failed: {:?}", payout_result.err());
            
            // Expected calculation:
            // Base pension: (60000 / 100) * 20 * 2 = 600 * 20 * 2 = 12000 * 2 = 24000
            // Total before tax: 24000 (base) + 10000 (insurance) = 34000
            // Tax amount: 34000 * 10 / 100 = 3400
            // Final payout: 34000 - 3400 = 30600
            let expected_payout: Balance = 30600;
            assert_eq!(payout_result.unwrap(), expected_payout);
        }

        #[ink::test]
        fn get_my_future_payout_no_tax_no_insurance() {
            let accounts = default_accounts();
            set_caller(accounts.alice);
            let mut contract = PensionManager::new();
            assert_eq!(contract.register_company(accounts.django), Ok(()));
            
            set_caller(accounts.django);
            let salary_bob: Balance = 50000;
            let years_bob: u32 = 10;
            assert_eq!(contract.update_pensioner_employment(accounts.bob, years_bob, salary_bob, EmploymentStatus::Active), Ok(()));

            set_caller(accounts.bob);
            let payout_result = contract.get_my_future_payout();
            // Expected: (50000 / 100) * 10 * 2 = 500 * 10 * 2 = 10000
            assert_eq!(payout_result.unwrap(), 10000);
        }

        #[ink::test]
        fn get_my_future_payout_deceased() {
            let accounts = default_accounts();
            set_caller(accounts.alice);
            let mut contract = PensionManager::new();
            assert_eq!(contract.register_company(accounts.django), Ok(()));

            set_caller(accounts.django);
            assert_eq!(contract.update_pensioner_employment(accounts.bob, 10, 50000, EmploymentStatus::Active), Ok(()));
            
            // Mark Bob as deceased (need a way to do this, for now manually edit storage or add a message)
            // For this test, let's assume a message `mark_deceased` exists and is called by owner.
            // Since it doesn't, we'll update the record and then try to get payout.
            let mut bob_data = contract.pensioners.get(&accounts.bob).unwrap();
            bob_data.is_deceased = true;
            contract.pensioners.insert(accounts.bob, &bob_data);

            set_caller(accounts.bob);
            assert_eq!(contract.get_my_future_payout(), Err(Error::PayoutNotApplicable));
        }
         #[ink::test]
        fn get_my_future_payout_pensioner_not_found() {
            let accounts = default_accounts();
            set_caller(accounts.alice);
            let contract = PensionManager::new();

            set_caller(accounts.bob); // Bob has no record
            assert_eq!(contract.get_my_future_payout(), Err(Error::PensionerNotFound));
        }

        #[ink::test]
        fn get_my_future_payout_invalid_tax_rate_in_storage() {
            let accounts = default_accounts();
            set_caller(accounts.alice);
            let mut contract = PensionManager::new();
            assert_eq!(contract.register_company(accounts.django), Ok(()));
            assert_eq!(contract.register_tax_office(accounts.frank), Ok(()));
            
            set_caller(accounts.django);
            assert_eq!(contract.update_pensioner_employment(accounts.bob, 10, 50000, EmploymentStatus::Active), Ok(()));

            // Manually insert invalid tax data (bypassing apply_pension_tax_rate check for testing payout robustness)
            let invalid_tax_info = TaxOfficeInfo { tax_office_id: accounts.frank, tax_rate_percentage: 150 };
            contract.pensioner_tax_config.insert(accounts.bob, &invalid_tax_info);

            set_caller(accounts.bob);
            assert_eq!(contract.get_my_future_payout(), Err(Error::InvalidInput));
        }

        #[ink::test]
        fn set_age_eligibility_works() {
            let accounts = default_accounts();
            set_caller(accounts.alice); // Owner
            let mut contract = PensionManager::new();

            // Register company and add pensioner
            assert_eq!(contract.register_company(accounts.django), Ok(()));
            set_caller(accounts.django);
            assert_eq!(contract.update_pensioner_employment(accounts.bob, 10, 50000, EmploymentStatus::Active), Ok(()));
            
            // Owner sets eligibility
            set_caller(accounts.alice);
            assert_eq!(contract.set_age_eligibility_status(accounts.bob, true), Ok(()));
            let bob_data = contract.get_pensioner_data(accounts.bob).unwrap();
            assert!(bob_data.is_eligible_for_payout_age_wise);

            // Non-owner tries to set - should fail
            set_caller(accounts.django); // Django is not owner
            assert_eq!(contract.set_age_eligibility_status(accounts.bob, false), Err(Error::Unauthorized));
        }

        #[ink::test]
        fn initiate_pension_payout_flow() {
            let accounts = default_accounts();
            set_caller(accounts.alice); // Owner
            let mut contract = PensionManager::new();

            // Setup: company, pensioner, eligibility
            assert_eq!(contract.register_company(accounts.django), Ok(()));
            set_caller(accounts.django);
            assert_eq!(contract.update_pensioner_employment(accounts.bob, 25, 70000, EmploymentStatus::Active), Ok(()));
            
            // Bob tries to initiate, should fail (not eligible by age)
            set_caller(accounts.bob);
            assert_eq!(contract.initiate_pension_payout(), Err(Error::NotYetEligibleForPayout));

            // Owner sets eligibility
            set_caller(accounts.alice);
            assert_eq!(contract.set_age_eligibility_status(accounts.bob, true), Ok(()));

            // Bob initiates payout
            set_caller(accounts.bob);
            let payout_result = contract.initiate_pension_payout();
            assert!(payout_result.is_ok());
            let expected_payout = (70000 / 100) * 25 * 2; // 35000
            assert_eq!(payout_result.unwrap(), expected_payout);

            let bob_data = contract.get_pensioner_data(accounts.bob).unwrap();
            assert!(bob_data.is_receiving_pension);
            assert_eq!(bob_data.pension_payout_amount, Some(expected_payout));

            // Try to initiate again
            assert_eq!(contract.initiate_pension_payout(), Err(Error::PayoutNotApplicable));
        }
        
        #[ink::test]
        fn designate_spouse_beneficiary_works() {
            let accounts = default_accounts();
            set_caller(accounts.alice); // Owner
            let mut contract = PensionManager::new();
            assert_eq!(contract.register_company(accounts.django), Ok(()));
            set_caller(accounts.django);
            assert_eq!(contract.update_pensioner_employment(accounts.bob, 10, 50000, EmploymentStatus::Active), Ok(()));

            set_caller(accounts.bob); // Bob designates spouse
            assert_eq!(contract.designate_spouse_beneficiary(accounts.eve), Ok(()));
            let bob_data = contract.get_pensioner_data(accounts.bob).unwrap();
            assert_eq!(bob_data.spouse_beneficiary, Some(accounts.eve));
        }

        #[ink::test]
        fn report_death_and_assign_spouse_benefit_works() {
            let accounts = default_accounts();
            set_caller(accounts.alice); // Owner
            let mut contract = PensionManager::new();

            // Setup: company, pensioner, eligibility, spouse
            assert_eq!(contract.register_company(accounts.django), Ok(()));
            set_caller(accounts.django); // Django (company)
            assert_eq!(contract.update_pensioner_employment(accounts.bob, 30, 100000, EmploymentStatus::Active), Ok(()));
            
            set_caller(accounts.bob); // Bob (pensioner)
            assert_eq!(contract.designate_spouse_beneficiary(accounts.eve), Ok(())); // Eve is spouse

            // Report death (can be anyone, e.g. Alice the owner)
            set_caller(accounts.alice);
            let benefit_result = contract.report_death_and_assign_spouse_benefit(accounts.bob);
            assert!(benefit_result.is_ok());
            
            let expected_pension = (100000 / 100) * 30 * 2; // 60000
            let expected_spouse_benefit = expected_pension * 20 / 100; // 12000
            assert_eq!(benefit_result.unwrap(), Some(expected_spouse_benefit));

            let bob_data = contract.get_pensioner_data(accounts.bob).unwrap();
            assert!(bob_data.is_deceased);
            assert!(!bob_data.is_receiving_pension);

            // Check spouse benefit stored
            set_caller(accounts.eve); // Eve (spouse)
            assert_eq!(contract.get_my_spouse_death_benefit(), Some(expected_spouse_benefit));

            // Try reporting death again
            set_caller(accounts.alice);
             assert_eq!(contract.report_death_and_assign_spouse_benefit(accounts.bob), Err(Error::AlreadyDeceased));
        }

        #[ink::test]
        fn report_death_no_spouse() {
            let accounts = default_accounts();
            set_caller(accounts.alice);
            let mut contract = PensionManager::new();
             assert_eq!(contract.register_company(accounts.django), Ok(()));
            set_caller(accounts.django);
            assert_eq!(contract.update_pensioner_employment(accounts.bob, 30, 100000, EmploymentStatus::Active), Ok(()));

            set_caller(accounts.alice); // Alice reports death
            let benefit_result = contract.report_death_and_assign_spouse_benefit(accounts.bob);
            assert_eq!(benefit_result, Ok(None)); // No spouse, so None benefit

            let bob_data = contract.get_pensioner_data(accounts.bob).unwrap();
            assert!(bob_data.is_deceased);
        }

         #[ink::test]
        fn update_pensioner_employment_initializes_new_fields() {
            let accounts = default_accounts();
            set_caller(accounts.alice);
            let mut contract = PensionManager::new();
            assert_eq!(contract.register_company(accounts.django), Ok(()));

            set_caller(accounts.django);
            contract.update_pensioner_employment(accounts.bob, 5, 50000, EmploymentStatus::Active).unwrap();
            
            let pensioner_data = contract.get_pensioner_data(accounts.bob).unwrap();
            assert_eq!(pensioner_data.is_eligible_for_payout_age_wise, false);
            assert_eq!(pensioner_data.pension_payout_amount, None);
            assert_eq!(pensioner_data.is_eligible_for_payout_age_wise, false);
            assert_eq!(pensioner_data.pension_payout_amount, None);
            assert_eq!(pensioner_data.spouse_beneficiary, None);
        }

        #[ink::test]
        fn set_age_eligibility_status_pensioner_not_found() {
            let accounts = default_accounts();
            set_caller(accounts.alice); // Owner
            let mut contract = PensionManager::new();

            assert_eq!(
                contract.set_age_eligibility_status(accounts.bob, true),
                Err(Error::PensionerNotFound)
            );
        }

        #[ink::test]
        fn apply_pension_tax_rate_unauthorized_and_not_found() {
            let accounts = default_accounts();
            set_caller(accounts.alice); // Owner
            let mut contract = PensionManager::new();

            // Register company and add pensioner
            assert_eq!(contract.register_company(accounts.django), Ok(()));
            set_caller(accounts.django);
            assert_eq!(contract.update_pensioner_employment(accounts.bob, 10, 50000, EmploymentStatus::Active), Ok(()));

            // Try as non-tax-office (e.g., company Django)
            set_caller(accounts.django);
            assert_eq!(
                contract.apply_pension_tax_rate(accounts.bob, 10),
                Err(Error::Unauthorized)
            );

            // Register tax office
            set_caller(accounts.alice);
            assert_eq!(contract.register_tax_office(accounts.frank), Ok(()));
            
            // Try for non-existent pensioner by authorized tax office
            set_caller(accounts.frank);
            assert_eq!(
                contract.apply_pension_tax_rate(accounts.charlie, 10), // Charlie is not a pensioner
                Err(Error::PensionerNotFound)
            );
        }

        #[ink::test]
        fn designate_spouse_beneficiary_error_cases() {
            let accounts = default_accounts();
            set_caller(accounts.alice); // Owner
            let mut contract = PensionManager::new();

            // Pensioner not found
            set_caller(accounts.bob); // Bob is not yet a pensioner
            assert_eq!(
                contract.designate_spouse_beneficiary(accounts.eve),
                Err(Error::PensionerNotFound)
            );

            // Setup Bob as pensioner
            assert_eq!(contract.register_company(accounts.django), Ok(()));
            set_caller(accounts.django);
            assert_eq!(contract.update_pensioner_employment(accounts.bob, 10, 50000, EmploymentStatus::Active), Ok(()));

            // Mark Bob as deceased by owner
            set_caller(accounts.alice);
            let mut bob_data = contract.pensioners.get_mut(&accounts.bob).unwrap();
            bob_data.is_deceased = true;
            contract.pensioners.insert(accounts.bob, &bob_data); // Manually update for test setup

            // Try to designate spouse when deceased
            set_caller(accounts.bob);
            assert_eq!(
                contract.designate_spouse_beneficiary(accounts.eve),
                Err(Error::PayoutNotApplicable) // PayoutNotApplicable is used for deceased state
            );
        }
        
        #[ink::test]
        fn initiate_pension_payout_pensioner_not_found() {
            let accounts = default_accounts();
            set_caller(accounts.bob); // Bob is not a pensioner
            let mut contract = PensionManager::new(); // Alice is owner by default if not set otherwise for constructor

            assert_eq!(
                contract.initiate_pension_payout(),
                Err(Error::PensionerNotFound)
            );
        }

        #[ink::test]
        fn unregister_entities_not_registered() {
            let accounts = default_accounts();
            set_caller(accounts.alice);
            let mut contract = PensionManager::new();

            assert_eq!(contract.unregister_company(accounts.django), Err(Error::NotRegistered));
            assert_eq!(contract.unregister_bank(accounts.eve), Err(Error::NotRegistered));
            assert_eq!(contract.unregister_tax_office(accounts.frank), Err(Error::NotRegistered));
        }
    }
}
