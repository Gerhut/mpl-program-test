use std::error::Error;

use solana_program_test::{processor, tokio, ProgramTest};
use solana_sdk::{
    program_pack::Pack,
    signer::{keypair::Keypair, Signer},
    transaction::Transaction,
};

#[cfg(feature = "test-bpf")]
#[tokio::test]
async fn test_create_master_edition() -> Result<(), Box<dyn Error>> {
    use solana_program::system_instruction;

    let mut program = ProgramTest::new("mpl_program_test", mpl_program_test::id(), None);
    program.add_program(
        "mpl_token_metadata",
        mpl_token_metadata::id(),
        processor!(mpl_token_metadata::processor::process_instruction),
    );
    let context = program.start_with_context().await;

    let rent = context.genesis_config().rent;
    let mint = Keypair::new();
    let creator = Keypair::new();
    let creator_wallet = Keypair::new();
    let (metadata_account, _bump_seed) = mpl_token_metadata::pda::find_metadata_account(&mint.pubkey());
    let transaction = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &context.payer.pubkey(),
                &mint.pubkey(),
                rent.minimum_balance(spl_token::state::Mint::LEN),
                spl_token::state::Mint::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_mint2(
                &spl_token::id(),
                &mint.pubkey(),
                &creator.pubkey(),
                None,
                0,
            )?,
            system_instruction::create_account(
                &context.payer.pubkey(),
                &creator_wallet.pubkey(),
                rent.minimum_balance(spl_token::state::Mint::LEN),
                spl_token::state::Account::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_account3(
                &spl_token::id(),
                &creator_wallet.pubkey(),
                &mint.pubkey(),
                &creator.pubkey(),
            )?,
            spl_token::instruction::mint_to(
                &spl_token::id(),
                &mint.pubkey(),
                &creator.pubkey(),
                &creator.pubkey(),
                &[],
                1,
            )?,
            mpl_token_metadata::instruction::create_metadata_accounts_v2(
                mpl_token_metadata::id(),
                metadata_account,
                mint.pubkey(),
                creator.pubkey(),
                context.payer.pubkey(),
                creator.pubkey(),
                "name".into(),
                "SYMBOL".into(),
                "https://example.com/".into(),
                None,
                0,
                true,
                false,
                None,
                None,
            ),
        ],
        None,
        &[&context.payer, &mint, &creator],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction(transaction)
        .await?;

    Ok(())
}
