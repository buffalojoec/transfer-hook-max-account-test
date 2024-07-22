// #![cfg(feature = "test-sbf")]

mod setup;

use {
    setup::{
        setup_extra_metas_account, setup_mint, setup_token_account,
        transfer_with_extra_metas_instruction,
    },
    solana_program_test::*,
    solana_sdk::{
        account::{Account, AccountSharedData},
        pubkey::Pubkey,
        signature::Keypair,
        signer::Signer,
        transaction::Transaction,
    },
    spl_tlv_account_resolution::account::ExtraAccountMeta,
    test_case::test_case,
};

// Fixed address of the counter account.
const COUNTER_ADDRESS: Pubkey =
    solana_program::pubkey!("Counter111111111111111111111111111111111111");

// Set up the program test.
fn setup() -> ProgramTest {
    ProgramTest::new(
        "transfer_hook_max_account_test",
        transfer_hook_max_account_test::id(),
        processor!(transfer_hook_max_account_test::process),
    )
}

// The counter extra meta.
fn counter_extra_meta() -> ExtraAccountMeta {
    ExtraAccountMeta::new_with_pubkey(&COUNTER_ADDRESS, false, false).unwrap()
}

// Generate N extra metas with pubkeys.
fn generate_extra_metas_with_pubkeys(pubkeys: &[Pubkey]) -> Vec<ExtraAccountMeta> {
    let mut extra_metas = vec![counter_extra_meta()];
    pubkeys.iter().for_each(|pubkey| {
        extra_metas.push(ExtraAccountMeta::new_with_pubkey(pubkey, false, false).unwrap())
    });
    extra_metas
}

#[test_case(3)]
#[test_case(4)]
#[test_case(5)]
#[test_case(6)]
#[test_case(7)]
#[test_case(8)]
#[test_case(9)]
#[test_case(10)]
#[test_case(11)]
#[test_case(12)]
#[test_case(13)]
#[test_case(14)]
#[test_case(15)]
// #[test_case(16)] // Nope!
#[tokio::test]
async fn test(num_keys: u8) {
    let mut context = setup().start_with_context().await;

    // Accounts for SPL Transfer Hook interface `Execute`.
    //
    // 0. `[ ]` Source token account.
    // 1. `[ ]` Token mint.
    // 2. `[ ]` Destination token account.
    // 3. `[ ]` Source owner.
    // 4. `[ ]` Extra account metas account.
    // 5..N     Extra account metas.
    let source_token_account = Pubkey::new_unique();
    let token_mint = Pubkey::new_unique();
    let destination_token_account = Pubkey::new_unique();
    let source_owner = Keypair::new();

    let pubkeys = vec![Pubkey::new_unique(); (num_keys - 1) as usize];
    let extra_metas = generate_extra_metas_with_pubkeys(&pubkeys);

    setup_mint(&mut context, &token_mint);
    setup_token_account(
        &mut context,
        &source_token_account,
        &source_owner.pubkey(),
        &token_mint,
        100,
    );
    setup_token_account(
        &mut context,
        &destination_token_account,
        &Pubkey::new_unique(),
        &token_mint,
        100,
    );
    setup_extra_metas_account(&mut context, &token_mint, &extra_metas);

    // Set up the counter.
    context.set_account(
        &COUNTER_ADDRESS,
        &AccountSharedData::from(Account {
            lamports: 100_000_000,
            data: vec![extra_metas.len() as u8],
            ..Account::default()
        }),
    );
    // Arbitrarily set up the remaining extra metas with some state.
    // Using mint size (85) as an example.
    pubkeys.iter().for_each(|pubkey| {
        context.set_account(
            pubkey,
            &AccountSharedData::from(Account {
                lamports: 100_000_000,
                data: vec![85; 85],
                ..Account::default()
            }),
        );
    });

    let instruction = transfer_with_extra_metas_instruction(
        &mut context,
        &source_token_account,
        &token_mint,
        &destination_token_account,
        &source_owner.pubkey(),
        10,
    )
    .await;

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer, &source_owner],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();
}
