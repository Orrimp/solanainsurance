//! Instruction processing logic for the pension insurance program

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{clock::Clock, rent::Rent, Sysvar},
    log::{sol_log, sol_log_64, sol_log_compute_units, sol_log_data, sol_log_params, sol_log_slice},
};

use crate::{
    errors::PensionError,
    instructions::PensionInstruction,
    state::{PensionAccount, PensionStatus, YearPointsEntry, MAX_POINTS_ENTRIES},
};

/// Process an instruction for the pension insurance program
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = PensionInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    #[cfg(feature = "debug")]
    {
        sol_log("Starting instruction processing");
        sol_log_params(accounts, instruction_data);
    }

    let result = match instruction {
        PensionInstruction::InitializePensioner { pensioner_pubkey, monthly_payment } => process_initialize_pensioner(program_id, accounts, pensioner_pubkey, monthly_payment),
        PensionInstruction::MarkDeceased => process_mark_deceased(program_id, accounts),
        PensionInstruction::CalculateDuePayment => process_calculate_due_payment(program_id, accounts),
        PensionInstruction::AddPoints { year, month, points } => process_add_year_month_points(program_id, accounts, year, month, points),
        PensionInstruction::GetPoints { year } => process_get_year_points(program_id, accounts, year),
        PensionInstruction::GetAllPoints {} => process_get_all_points(program_id, accounts),
        PensionInstruction::StartPayout { recipient } => process_start_payout(program_id, accounts, recipient),
        PensionInstruction::StopPayout => process_stop_payout(program_id, accounts),
        PensionInstruction::ChangePayoutRecipient { new_recipient } => process_change_payout_recipient(program_id, accounts, new_recipient),
        PensionInstruction::RecalculateMonthlyFromPoints { base_lamports, point_multiplier_lamports } => process_recalculate_monthly_from_points(program_id, accounts, base_lamports, point_multiplier_lamports),
        PensionInstruction::Contribute { lamports, points, year } => process_contribute(program_id, accounts, lamports, points, year),
        PensionInstruction::StartPayoutPeriod => process_start_payout_period(program_id, accounts),
        PensionInstruction::WithdrawMonthly => process_withdraw_monthly(program_id, accounts),
    };

    #[cfg(feature = "debug")]
    {
        sol_log("Instruction processing finished.");
        sol_log_compute_units();
    }

    result
}

/// Initialize a new pension account for a pensioner
pub fn process_initialize_pensioner(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    pensioner_pubkey: Pubkey,
    monthly_payment: u64,
) -> ProgramResult {
    #[cfg(feature = "debug")]
    msg!("process_initialize_pensioner");

    let accounts_iter = &mut accounts.iter();
    
    let pension_account = next_account_info(accounts_iter)?;
    let authority_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    // Verify accounts
    if !authority_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !pension_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Calculate rent
    let rent = Rent::get()?;
    let required_lamports = rent.minimum_balance(PensionAccount::serialized_size());

    // Create the account
    invoke(
        &system_instruction::create_account(
            authority_account.key,
            pension_account.key,
            required_lamports,
            PensionAccount::serialized_size() as u64,
            program_id,
        ),
        &[
            authority_account.clone(),
            pension_account.clone(),
            system_program.clone(),
        ],
    )?;

    // Initialize the pension account data
    let pension_data = PensionAccount {
        authority: *authority_account.key,
        pensioner: pensioner_pubkey,
        status: PensionStatus::Active,
        monthly_payment,
        last_payment_timestamp: Clock::get()?.unix_timestamp,
        payout_enabled: 0,
        payout_recipient: Pubkey::default(),
        points_count: 0,
        points: [YearPointsEntry { year: 0, month: 0, points: 0 }; MAX_POINTS_ENTRIES],
        total_contributions_lamports: 0,
        date_of_birth: todo!(),
        metadata: todo!(),
        date_of_retirement: todo!(),
        date_of_death: todo!(),
        relations: todo!(),
    };

    // Serialize the data into the account
    let mut account_data = &mut pension_account.data.borrow_mut()[..];
    pension_data.serialize(&mut account_data)?;

    msg!("Initialized new pension account for: {}", pensioner_pubkey);
    #[cfg(feature = "debug")]
    sol_log_compute_units();
    Ok(())
}

/// Mark a pensioner as deceased to stop payments
pub fn process_mark_deceased(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    #[cfg(feature = "debug")]
    msg!("process_mark_deceased");
    let accounts_iter = &mut accounts.iter();
    
    let pension_account = next_account_info(accounts_iter)?;
    let authority_account = next_account_info(accounts_iter)?;

    // Verify accounts
    if pension_account.owner != program_id {
        return Err(PensionError::IncorrectOwner.into());
    }

    if !authority_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Deserialize the pension account data
    let mut data = pension_account.data.borrow_mut();
    let mut pension_data: PensionAccount = PensionAccount::try_from_slice(&data)?;

    // Verify authority
    if pension_data.authority != *authority_account.key {
        return Err(PensionError::InvalidAuthority.into());
    }

    // Check if already deceased
    if pension_data.status == PensionStatus::Deceased {
        return Err(PensionError::AlreadyDeceased.into());
    }

    // Update status
    pension_data.status = PensionStatus::Deceased;

    // Serialize back to account
    pension_data.serialize(&mut &mut data[..])?;

    msg!(
        "Pensioner {} marked as deceased. Payments will be stopped.",
        pension_data.pensioner
    );
    #[cfg(feature = "debug")]
    sol_log_compute_units();
    Ok(())
}

/// Calculate and log the payment currently due to the pensioner
pub fn process_calculate_due_payment(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    #[cfg(feature = "debug")]
    msg!("process_calculate_due_payment");
    let accounts_iter = &mut accounts.iter();
    
    let pension_account = next_account_info(accounts_iter)?;

    // Verify the pension account is owned by our program
    if pension_account.owner != program_id {
        return Err(PensionError::IncorrectOwner.into());
    }

    // Deserialize the pension account data
    let data = pension_account.data.borrow();
    let pension_data: PensionAccount = PensionAccount::try_from_slice(&data)?;

    // Check if pensioner is active
    if pension_data.status != PensionStatus::Active {
        return Err(PensionError::PensionerNotActive.into());
    }

    let current_timestamp = Clock::get()?.unix_timestamp;
    let time_since_last_payment =
        current_timestamp.saturating_sub(pension_data.last_payment_timestamp);

    // Average seconds in a month (30.44 days based on Gregorian calendar)
    // 365.25 days per year ÷ 12 months = 30.4375 days per month
    // 30.4375 days × 24 hours × 60 minutes × 60 seconds = 2,629,746 seconds
    const SECONDS_IN_MONTH: i64 = 2_629_746;

    if time_since_last_payment > 0 {
        let months_due = time_since_last_payment / SECONDS_IN_MONTH;
        if months_due > 0 {
            let total_due = (months_due as u64).saturating_mul(pension_data.monthly_payment);
            msg!("Calculated due payment: {} lamports", total_due);
        } else {
            msg!("No full month has passed. Due payment: 0 lamports");
        }
    } else {
        msg!("No time has passed since last payment. Due payment: 0 lamports");
    }

    #[cfg(feature = "debug")]
    sol_log_compute_units();
    Ok(())
}

/// Add year points
pub fn process_add_year_month_points(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    year: u16,
    month: u16,
    points: u64,
) -> ProgramResult {
    #[cfg(feature = "debug")]
    msg!("process_add_year_month_points");
    let accounts_iter = &mut accounts.iter();
    let pension_account = next_account_info(accounts_iter)?;
    let authority_account = next_account_info(accounts_iter)?;
    if pension_account.owner != program_id { return Err(PensionError::IncorrectOwner.into()); }
    if !authority_account.is_signer { return Err(ProgramError::MissingRequiredSignature); }
    let mut data = pension_account.data.borrow_mut();
    let mut pension_data: PensionAccount = PensionAccount::try_from_slice(&data)?;
    if pension_data.authority != *authority_account.key { return Err(PensionError::InvalidAuthority.into()); }
    if pension_data.points_count as usize >= MAX_POINTS_ENTRIES { return Err(PensionError::PointsCapacityExceeded.into()); }
    // Check duplicate year
    for i in 0..pension_data.points_count as usize {
        if pension_data.points[i].year == year && pension_data.points[i].month == month { return Err(PensionError::YearAlreadyExists.into()); }
    }
    let idx = pension_data.points_count as usize;
    pension_data.points[idx].year = year;
    pension_data.points[idx].month = month;
    pension_data.points[idx].points = points;
    pension_data.points_count += 1;
    pension_data.serialize(&mut &mut data[..])?;
    msg!("Added points year={} month={} points={}", year, month, points);
    #[cfg(feature = "debug")]
    sol_log_compute_units();
    Ok(())
}

/// Get year points (log only)
pub fn process_get_year_points(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    year: u16,
) -> ProgramResult {
    #[cfg(feature = "debug")]
    msg!("process_get_year_points");
    let accounts_iter = &mut accounts.iter();
    let pension_account = next_account_info(accounts_iter)?;
    if pension_account.owner != program_id { return Err(PensionError::IncorrectOwner.into()); }
    let data = pension_account.data.borrow();
    let pension_data: PensionAccount = PensionAccount::try_from_slice(&data)?;
    for i in 0..pension_data.points_count as usize {
        if pension_data.points[i].year == year {
            msg!("Points for year {}: {}", year, pension_data.points[i].points);
            #[cfg(feature = "debug")]
            sol_log_compute_units();
            return Ok(());
        }
    }
    Err(PensionError::YearNotFound.into())
}

/// Get all points (log only)
pub fn process_get_all_points(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    #[cfg(feature = "debug")]
    msg!("process_get_all_points");
    let accounts_iter = &mut accounts.iter();
    let pension_account = next_account_info(accounts_iter)?;
    if pension_account.owner != program_id { return Err(PensionError::IncorrectOwner.into()); }
    let data = pension_account.data.borrow();
    let pension_data: PensionAccount = PensionAccount::try_from_slice(&data)?;
    msg!("Pension points history:");
    for i in 0..pension_data.points_count as usize {
        msg!("Year: {}, Month: {}, Points: {}", pension_data.points[i].year, pension_data.points[i].month, pension_data.points[i].points);
    }
    #[cfg(feature = "debug")]
    sol_log_compute_units();
    Ok(())
}

/// Start payout
pub fn process_start_payout(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    recipient: Pubkey,
) -> ProgramResult {
    #[cfg(feature = "debug")]
    msg!("process_start_payout");
    let accounts_iter = &mut accounts.iter();
    let pension_account = next_account_info(accounts_iter)?;
    let authority_account = next_account_info(accounts_iter)?;
    let _recipient_account = next_account_info(accounts_iter)?; // readonly check placeholder
    if pension_account.owner != program_id { return Err(PensionError::IncorrectOwner.into()); }
    if !authority_account.is_signer { return Err(ProgramError::MissingRequiredSignature); }
    let mut data = pension_account.data.borrow_mut();
    let mut pension_data: PensionAccount = PensionAccount::try_from_slice(&data)?;
    if pension_data.authority != *authority_account.key { return Err(PensionError::InvalidAuthority.into()); }
    if pension_data.payout_enabled == 1 { return Err(PensionError::PayoutAlreadyActive.into()); }
    pension_data.payout_enabled = 1;
    pension_data.payout_recipient = recipient;
    pension_data.serialize(&mut &mut data[..])?;
    msg!("Payout started to {}", recipient);
    #[cfg(feature = "debug")]
    sol_log_compute_units();
    Ok(())
}

/// Stop payout
pub fn process_stop_payout(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    #[cfg(feature = "debug")]
    msg!("process_stop_payout");
    let accounts_iter = &mut accounts.iter();
    let pension_account = next_account_info(accounts_iter)?;
    let authority_account = next_account_info(accounts_iter)?;
    if pension_account.owner != program_id { return Err(PensionError::IncorrectOwner.into()); }
    if !authority_account.is_signer { return Err(ProgramError::MissingRequiredSignature); }
    let mut data = pension_account.data.borrow_mut();
    let mut pension_data: PensionAccount = PensionAccount::try_from_slice(&data)?;
    if pension_data.authority != *authority_account.key { return Err(PensionError::InvalidAuthority.into()); }
    if pension_data.payout_enabled == 0 { return Err(PensionError::PayoutNotActive.into()); }
    pension_data.payout_enabled = 0;
    pension_data.payout_recipient = Pubkey::default();
    pension_data.serialize(&mut &mut data[..])?;
    msg!("Payout stopped");
    #[cfg(feature = "debug")]
    sol_log_compute_units();
    Ok(())
}

/// Change payout recipient
pub fn process_change_payout_recipient(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    new_recipient: Pubkey,
) -> ProgramResult {
    #[cfg(feature = "debug")]
    msg!("process_change_payout_recipient");
    let accounts_iter = &mut accounts.iter();
    let pension_account = next_account_info(accounts_iter)?;
    let authority_account = next_account_info(accounts_iter)?;
    let _new_recipient_account = next_account_info(accounts_iter)?;
    if pension_account.owner != program_id { return Err(PensionError::IncorrectOwner.into()); }
    if !authority_account.is_signer { return Err(ProgramError::MissingRequiredSignature); }
    let mut data = pension_account.data.borrow_mut();
    let mut pension_data: PensionAccount = PensionAccount::try_from_slice(&data)?;
    if pension_data.authority != *authority_account.key { return Err(PensionError::InvalidAuthority.into()); }
    if pension_data.payout_enabled == 0 { return Err(PensionError::PayoutNotActive.into()); }
    pension_data.payout_recipient = new_recipient;
    pension_data.serialize(&mut &mut data[..])?;
    msg!("Payout recipient changed to {}", new_recipient);
    #[cfg(feature = "debug")]
    sol_log_compute_units();
    Ok(())
}

/// Recalculate monthly payment based on total points
pub fn process_recalculate_monthly_from_points(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    base_lamports: u64,
    point_multiplier_lamports: u64,
) -> ProgramResult {
    #[cfg(feature = "debug")]
    msg!("process_recalculate_monthly_from_points");
    let accounts_iter = &mut accounts.iter();
    let pension_account = next_account_info(accounts_iter)?;
    let authority_account = next_account_info(accounts_iter)?;
    if pension_account.owner != program_id { return Err(PensionError::IncorrectOwner.into()); }
    if !authority_account.is_signer { return Err(ProgramError::MissingRequiredSignature); }
    let mut data = pension_account.data.borrow_mut();
    let mut pension_data: PensionAccount = PensionAccount::try_from_slice(&data)?;
    if pension_data.authority != *authority_account.key { return Err(PensionError::InvalidAuthority.into()); }
    let mut total_points = 0u64;
    for i in 0..pension_data.points_count as usize { total_points = total_points.saturating_add(pension_data.points[i].points); }
    let points_component = total_points.saturating_mul(point_multiplier_lamports);
    let old = pension_data.monthly_payment;
    pension_data.monthly_payment = base_lamports.saturating_add(points_component);
    pension_data.serialize(&mut &mut data[..])?;
    msg!("Recalculated monthly payment old={} new={} base={} points_component={} total_points={} point_multiplier={}", old, pension_data.monthly_payment, base_lamports, points_component, total_points, point_multiplier_lamports);
    #[cfg(feature = "debug")]
    sol_log_compute_units();
    Ok(())
}

/// Contribute lamports and optionally points
pub fn process_contribute(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    lamports: u64,
    points: u64,
    year: u16,
) -> ProgramResult {
    #[cfg(feature = "debug")]
    msg!("process_contribute");
    let accounts_iter = &mut accounts.iter();
    let pension_account = next_account_info(accounts_iter)?;
    let contributor = next_account_info(accounts_iter)?; // authority/org
    let system_program = next_account_info(accounts_iter)?;
    if pension_account.owner != program_id { return Err(PensionError::IncorrectOwner.into()); }
    if !contributor.is_signer { return Err(ProgramError::MissingRequiredSignature); }
    // Transfer lamports
    if lamports > 0 {
        invoke(
            &system_instruction::transfer(contributor.key, pension_account.key, lamports),
            &[contributor.clone(), pension_account.clone(), system_program.clone()],
        )?;
    }
    let mut data = pension_account.data.borrow_mut();
    let mut state: PensionAccount = PensionAccount::try_from_slice(&data)?;
    if state.authority != *contributor.key { /* allow other orgs? For now require authority */ return Err(PensionError::InvalidAuthority.into()); }
    state.total_contributions_lamports = state.total_contributions_lamports.saturating_add(lamports);
    if points > 0 {
        // add year points similar to process_add_year_points logic (no duplicate year check if points=0 skip)
        // check capacity
        if state.points_count as usize >= MAX_POINTS_ENTRIES { return Err(PensionError::PointsCapacityExceeded.into()); }
        for i in 0..state.points_count as usize { if state.points[i].year == year { return Err(PensionError::YearAlreadyExists.into()); } }
        let idx = state.points_count as usize;
        state.points[idx].year = year;
        state.points[idx].points = points;
        state.points_count += 1;
    }
    state.serialize(&mut &mut data[..])?;
    msg!("Contribution applied lamports={} points={} year={} total_contributions={}", lamports, points, year, state.total_contributions_lamports);
    #[cfg(feature = "debug")]
    sol_log_compute_units();
    Ok(())
}

/// Start payout period
pub fn process_start_payout_period(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    #[cfg(feature = "debug")]
    msg!("process_start_payout_period");
    let accounts_iter = &mut accounts.iter();
    let pension_account = next_account_info(accounts_iter)?;
    let authority = next_account_info(accounts_iter)?;
    if pension_account.owner != program_id { return Err(PensionError::IncorrectOwner.into()); }
    if !authority.is_signer { return Err(ProgramError::MissingRequiredSignature); }
    let mut data = pension_account.data.borrow_mut();
    let mut state: PensionAccount = PensionAccount::try_from_slice(&data)?;
    if state.authority != *authority.key { return Err(PensionError::InvalidAuthority.into()); }
    if state.payout_enabled == 1 { return Err(PensionError::PayoutAlreadyActive.into()); }
    state.payout_enabled = 1;
    state.serialize(&mut &mut data[..])?;
    msg!("Payout period started");
    #[cfg(feature = "debug")]
    sol_log_compute_units();
    Ok(())
}

/// Withdraw monthly payout from contributions balance
pub fn process_withdraw_monthly(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    #[cfg(feature = "debug")]
    msg!("process_withdraw_monthly");
    let accounts_iter = &mut accounts.iter();
    let pension_account = next_account_info(accounts_iter)?;
    let authority = next_account_info(accounts_iter)?;
    let pensioner = next_account_info(accounts_iter)?;
    if pension_account.owner != program_id { return Err(PensionError::IncorrectOwner.into()); }
    if !authority.is_signer || !pensioner.is_signer { return Err(ProgramError::MissingRequiredSignature); }
    let mut data = pension_account.data.borrow_mut();
    let mut state: PensionAccount = PensionAccount::try_from_slice(&data)?;
    if state.authority != *authority.key { return Err(PensionError::InvalidAuthority.into()); }
    if state.payout_enabled == 0 { return Err(PensionError::PayoutNotActive.into()); }
    if state.status != PensionStatus::Active { return Err(PensionError::PensionerNotActive.into()); }
    // ensure enough balance
    if state.total_contributions_lamports < state.monthly_payment { msg!("Insufficient contributions for monthly payout"); return Ok(()); }
    // simulate payout by decrementing contributions (actual transfer omitted for simplicity)
    state.total_contributions_lamports = state.total_contributions_lamports.saturating_sub(state.monthly_payment);
    state.last_payment_timestamp = Clock::get()?.unix_timestamp;
    state.serialize(&mut &mut data[..])?;
    msg!("Monthly payout simulated amount={} remaining_contributions={}", state.monthly_payment, state.total_contributions_lamports);
    #[cfg(feature = "debug")]
    sol_log_compute_units();
    Ok(())
}
