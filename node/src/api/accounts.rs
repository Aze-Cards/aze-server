use aze_lib::accounts::{create_basic_aze_game_account, create_basic_aze_player_account};
use aze_lib::client::{self, create_aze_client, AzeAccountTemplate, AzeClient, AzeGameMethods, AzeTransactionTemplate, SendCardTransactionData};
use aze_lib::notes::create_deal_note;
use miden_client::client::transactions::{PaymentTransactionData, TransactionTemplate};
use miden_lib::{transaction, AuthScheme};
use miden_objects::{
    accounts::{Account, AccountId, AccountStorage, StorageSlotType},
    assembly::ProgramAst,
    assets::{Asset, AssetVault, FungibleAsset},
    crypto::dsa::rpo_falcon512::{KeyPair, PublicKey},
    transaction::TransactionArgs,
    Felt, Word, ONE, ZERO,
};
use miden_objects::crypto::rand::RpoRandomCoin;
use miden_tx::TransactionExecutor;

use actix_web::{
    error::ResponseError,
    get,
    http::{header::ContentType, StatusCode},
    post, put,
    web::Data,
    web::Json,
    web::Path,
    HttpResponse,
};
use derive_more::Display;
use serde::{Deserialize, Serialize};
use miden_client::client::accounts::AccountStorageMode;
use miden_objects::accounts::ACCOUNT_ID_FUNGIBLE_FAUCET_ON_CHAIN;

use crate::model::player;

#[derive(Deserialize, Serialize)]
pub struct AccountCreationResponse {
    is_created: bool,
}

#[derive(Debug, Display)]
pub enum AccountCreationError {
    AccountCreationFailed,
    BadTaskRequest,
}

impl ResponseError for AccountCreationError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match self {
            AccountCreationError::AccountCreationFailed => StatusCode::FAILED_DEPENDENCY,
            AccountCreationError::BadTaskRequest => StatusCode::BAD_REQUEST,
        }
    }
}

#[get("/v1/game/create-account")]
pub async fn create_aze_game_account() -> Result<Json<AccountCreationResponse>, AccountCreationError>
{
    //it will get the player accounts id: Felt
    // then create the game account 
    // create notes so that players can consume the cards in it
    // after the dealing is done, the game id is returned

    let player_ids: Vec<Felt> = vec![Felt::new(1), Felt::new(2), Felt::new(3), Felt::new(4)];
    let player_account_ids: Vec<AccountId> = player_ids.iter().map(|felt| felt.into()).collect();
    
    let mut client: AzeClient = create_aze_client();

    let (game_account, _) = client
        .new_game_account(AzeAccountTemplate::GameAccount {
            mutable_code: false,
            storage_mode: AccountStorageMode::Local, // for now
        })
        .unwrap();
    let game_account_id = game_account.id();
    println!("Account created: {:?}", game_account_id);

    // Create an asset
    let faucet_id = AccountId::try_from(ACCOUNT_ID_FUNGIBLE_FAUCET_ON_CHAIN).unwrap();
    let fungible_asset: Asset = FungibleAsset::new(faucet_id, 100).unwrap().into();

    let sender_account_id = game_account_id;

    let sample_card = [Felt::new(99), Felt::new(99), Felt::new(99), Felt::new(99)];
    let cards = [sample_card; 8];
    for i in player_account_ids.len() {
        let target_account_id = player_account_ids[i].into();
        

        let input_cards = cards[i].into();
        let payment_txn_data = SendCardTransactionData::new(fungible_asset, sender_account_id, target_account_id, cards[i]);
        let transaction_template = AzeTransactionTemplate::SendCard(payment_txn_data);
        
        // let txn_result = client.new_transaction(transaction_template).unwrap();

        // new_aze_send_card_transaction(t)

        // client.send_transaction(txn_result).await.unwrap();
    }



    // println!("Account by this client {:?} ", client.get_accounts());
    // let val = game_account.storage().get_item(1);
    // println!("Account storage value: {:?}", val);

    // println!("Account created: {:?}", game_account);

    Ok(Json(AccountCreationResponse { is_created: true }))
}

#[get("/v1/player/create-account")]
pub async fn create_aze_player_account(
) -> Result<Json<AccountCreationResponse>, AccountCreationError> {
    use miden_objects::accounts::AccountType;
    let key_pair: KeyPair = KeyPair::new().unwrap();
    let pub_key: PublicKey = key_pair.public_key();
    let auth_scheme: AuthScheme = AuthScheme::RpoFalcon512 { pub_key };

    // we need to use an initial seed to create the wallet account
    let init_seed: [u8; 32] = [
        95, 113, 209, 94, 84, 105, 250, 242, 223, 203, 216, 124, 22, 159, 14, 132, 215, 85, 183,
        204, 149, 90, 166, 68, 100, 73, 106, 168, 125, 237, 138, 16,
    ];

    let (game_account, _) = create_basic_aze_player_account(
        init_seed,
        auth_scheme,
        AccountType::RegularAccountImmutableCode,
    )
    .unwrap();
    // println!("Account created: {:?}", game_account);

    Ok(Json(AccountCreationResponse { is_created: true }))
}
