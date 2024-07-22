// #![cfg(feature = "test-sbf")]

use {
    solana_program_test::*,
    solana_sdk::{
        account::{Account, AccountSharedData},
        instruction::Instruction,
        pubkey::Pubkey,
    },
    spl_tlv_account_resolution::{account::ExtraAccountMeta, state::ExtraAccountMetaList},
    spl_token_2022::{
        extension::{
            transfer_hook::{TransferHook, TransferHookAccount},
            BaseStateWithExtensionsMut, ExtensionType, StateWithExtensionsMut,
        },
        state::{Account as TokenAccount, AccountState, Mint},
    },
    spl_transfer_hook_interface::{
        get_extra_account_metas_address, instruction::ExecuteInstruction,
        offchain::add_extra_account_metas_for_execute,
    },
};

// Set up a mint with transfer hook extension.
pub fn setup_mint(context: &mut ProgramTestContext, mint: &Pubkey) {
    let account_size =
        ExtensionType::try_calculate_account_len::<Mint>(&[ExtensionType::TransferHook]).unwrap();
    let mut data = vec![0; account_size];
    {
        let mut state = StateWithExtensionsMut::<Mint>::unpack_uninitialized(&mut data).unwrap();
        state
            .init_extension::<TransferHook>(true)
            .unwrap()
            .program_id = Some(transfer_hook_max_account_test::id())
            .try_into()
            .unwrap();
        state.base = Mint {
            is_initialized: true,
            supply: 100_000,
            ..Mint::default()
        };
        state.pack_base();
        state.init_account_type().unwrap();
    }
    context.set_account(
        mint,
        &AccountSharedData::from(Account {
            lamports: 100_000_000,
            data,
            owner: spl_token_2022::id(),
            ..Account::default()
        }),
    );
}

// Set up a token account with transfer hook extension.
pub fn setup_token_account(
    context: &mut ProgramTestContext,
    token_account: &Pubkey,
    owner: &Pubkey,
    mint: &Pubkey,
    amount: u64,
) {
    let account_size = ExtensionType::try_calculate_account_len::<TokenAccount>(&[
        ExtensionType::TransferHookAccount,
    ])
    .unwrap();
    let mut data = vec![0; account_size];
    {
        let mut state =
            StateWithExtensionsMut::<TokenAccount>::unpack_uninitialized(&mut data).unwrap();
        state.init_extension::<TransferHookAccount>(true).unwrap();
        state.base = TokenAccount {
            amount,
            mint: *mint,
            owner: *owner,
            state: AccountState::Initialized,
            ..TokenAccount::default()
        };
        state.pack_base();
        state.init_account_type().unwrap();
    }
    context.set_account(
        token_account,
        &AccountSharedData::from(Account {
            lamports: 100_000_000,
            data,
            owner: spl_token_2022::id(),
            ..Account::default()
        }),
    );
}

// Set up a validation (extra metas) account.
pub fn setup_extra_metas_account(
    context: &mut ProgramTestContext,
    mint: &Pubkey,
    extra_metas: &[ExtraAccountMeta],
) {
    let address = get_extra_account_metas_address(mint, &transfer_hook_max_account_test::id());
    let data_len = ExtraAccountMetaList::size_of(extra_metas.len()).unwrap();
    let mut data = vec![0; data_len];
    ExtraAccountMetaList::init::<ExecuteInstruction>(&mut data, extra_metas).unwrap();
    context.set_account(
        &address,
        &AccountSharedData::from(Account {
            lamports: 100_000_000,
            data,
            owner: transfer_hook_max_account_test::id(),
            ..Account::default()
        }),
    );
}

// Create a `TransferChecked` instruction with extra account metas.
pub async fn transfer_with_extra_metas_instruction(
    context: &mut ProgramTestContext,
    source: &Pubkey,
    mint: &Pubkey,
    destination: &Pubkey,
    owner: &Pubkey,
    amount: u64,
) -> Instruction {
    let mut instruction = spl_token_2022::instruction::transfer_checked(
        &spl_token_2022::id(),
        source,
        mint,
        destination,
        owner,
        &[],
        amount,
        /* decimals */ 0,
    )
    .unwrap();
    {
        let extra_metas_address =
            get_extra_account_metas_address(mint, &transfer_hook_max_account_test::id());
        let extra_metas_account = context
            .banks_client
            .get_account(extra_metas_address)
            .await
            .unwrap()
            .unwrap();
        add_extra_account_metas_for_execute(
            &mut instruction,
            &transfer_hook_max_account_test::id(),
            source,
            mint,
            destination,
            owner,
            amount,
            |key| {
                let data = if key.eq(&extra_metas_address) {
                    Some(extra_metas_account.data.clone())
                } else {
                    None
                };
                async move { Ok(data) }
            },
        )
        .await
        .unwrap();
    }
    instruction
}
