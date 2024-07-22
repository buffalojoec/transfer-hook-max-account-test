use {
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        msg,
        program_error::ProgramError,
        pubkey::Pubkey,
    },
    spl_transfer_hook_interface::instruction::TransferHookInstruction,
};

solana_program::declare_id!("Program111111111111111111111111111111111111");

solana_program::entrypoint!(process);

pub fn process(_program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
    if let Ok(TransferHookInstruction::Execute { amount: _ }) =
        TransferHookInstruction::unpack(input)
    {
        process_transfer_hook(accounts)
    } else {
        Err(ProgramError::InvalidInstructionData)
    }
}

fn process_transfer_hook(accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    // Accounts for SPL Transfer Hook interface `Execute`.
    //
    // 0. `[ ]` Source token account.
    // 1. `[ ]` Token mint.
    // 2. `[ ]` Destination token account.
    // 3. `[ ]` Source owner.
    // 4. `[ ]` Extra account metas account.
    // 5..N     Extra account metas.
    let _source_token_account_info = next_account_info(accounts_iter)?;
    let _token_mint_info = next_account_info(accounts_iter)?;
    let _destination_token_account_info = next_account_info(accounts_iter)?;
    let _source_owner_info = next_account_info(accounts_iter)?;
    let _extra_account_metas_info = next_account_info(accounts_iter)?;

    // Read the first account (single byte) to see how many accounts were
    // configured as extra account metas.
    let counter_info = next_account_info(accounts_iter)?;
    let count = counter_info.try_borrow_data()?[0];

    // Read the extra account metas (counter already read).
    for _ in 1..count {
        let _extra_account_meta_info = next_account_info(accounts_iter)?;
    }

    msg!("Processed {} extra account metas", count);

    Ok(())
}
