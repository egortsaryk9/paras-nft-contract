use near_contract_standards::non_fungible_token::core::{
    NonFungibleTokenCore, NonFungibleTokenResolver,
};
use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata, NFT_METADATA_SPEC,
};
use near_contract_standards::non_fungible_token::NonFungibleToken;
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupSet, UnorderedMap, UnorderedSet};
use near_sdk::json_types::{ValidAccountId, U128, U64};
use near_sdk::{
    assert_one_yocto, env, near_bindgen, serde_json::json, AccountId, Balance, BorshStorageKey,
    PanicOnDefault, Promise, PromiseOrValue, Gas, ext_contract, Timestamp
};
use near_sdk::serde::{Deserialize, Serialize};
use std::collections::HashMap;
// use near_sdk::env::is_valid_account_id;

pub mod event;
pub use event::NearEvent;

/// between token_series_id and edition number e.g. 42:2 where 42 is series and 2 is edition
pub const TOKEN_DELIMETER: char = ':';
/// TokenMetadata.title returned for individual token e.g. "Title — 2/10" where 10 is max copies
pub const TITLE_DELIMETER: &str = " #";
/// e.g. "Title — 2/10" where 10 is max copies
pub const EDITION_DELIMETER: &str = "/";

const GAS_FOR_RESOLVE_TRANSFER: Gas = 10_000_000_000_000;
const GAS_FOR_NFT_TRANSFER_CALL: Gas = 30_000_000_000_000 + GAS_FOR_RESOLVE_TRANSFER;
// const GAS_FOR_NFT_APPROVE: Gas = 10_000_000_000_000;
// const GAS_FOR_MINT: Gas = 90_000_000_000_000;
const NO_DEPOSIT: Balance = 0;
// const MAX_PRICE: Balance = 1_000_000_000 * 10u128.pow(24);

pub const NFT_PRICE: u128 = 3_480_000_000_000_000_000_000_000;
pub const NFT_REGISTRATION_FEE: u128 = 20_000_000_000_000_000_000_000;
pub const NFT_TOTAL_PRICE: u128 = NFT_PRICE + NFT_REGISTRATION_FEE;


pub const NFT_MAX_SUPPLY: u128 = 26_800;

pub type TokenSeriesId = String;
pub type TimestampSec = u32;
pub type ContractAndTokenId = String;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Payout {
    pub payout: HashMap<AccountId, U128>,
}

#[ext_contract(ext_non_fungible_token_receiver)]
trait NonFungibleTokenReceiver {
    /// Returns `true` if the token should be returned back to the sender.
    fn nft_on_transfer(
        &mut self,
        sender_id: AccountId,
        previous_owner_id: AccountId,
        token_id: TokenId,
        msg: String,
    ) -> Promise;
}

#[ext_contract(ext_approval_receiver)]
pub trait NonFungibleTokenReceiver {
    fn nft_on_approve(
        &mut self,
        token_id: TokenId,
        owner_id: AccountId,
        approval_id: u64,
        msg: String,
    );
}

#[ext_contract(ext_self)]
trait NonFungibleTokenResolver {
    fn nft_resolve_transfer(
        &mut self,
        previous_owner_id: AccountId,
        receiver_id: AccountId,
        token_id: TokenId,
        approved_account_ids: Option<HashMap<AccountId, u64>>,
    ) -> bool;
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct TokenSeries {
	metadata: TokenMetadata,
	creator_id: AccountId,
	tokens: UnorderedSet<TokenId>,
    price: Option<Balance>,
    is_mintable: bool,
    royalty: HashMap<AccountId, u32>
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenSeriesJson {
    token_series_id: TokenSeriesId,
	metadata: TokenMetadata,
	creator_id: AccountId,
    royalty: HashMap<AccountId, u32>,
    transaction_fee: Option<U128>
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TransactionFee {
    pub next_fee: Option<u16>,
    pub start_time: Option<TimestampSec>,
    pub current_fee: u16,
}

#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct MarketDataTransactionFee {
    pub transaction_fee: UnorderedMap<TokenSeriesId, u128>
}

near_sdk::setup_alloc!();

// #[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
// pub struct ContractV1 {
//     tokens: NonFungibleToken,
//     metadata: LazyOption<NFTContractMetadata>,
//     // CUSTOM
// 	token_series_by_id: UnorderedMap<TokenSeriesId, TokenSeries>,
//     treasury_id: AccountId,
//     transaction_fee: TransactionFee
// }

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    tokens: NonFungibleToken,
    metadata: LazyOption<NFTContractMetadata>,
    // CUSTOM
    token_series_by_id: UnorderedMap<TokenSeriesId, TokenSeries>,
    treasury_id: AccountId,
    transaction_fee: TransactionFee,
    market_data_transaction_fee: MarketDataTransactionFee,
    token_metadata_admins: LookupSet<AccountId>,
    default_token_metadata: LazyOption<TokenMetadata>,
}

const DATA_IMAGE_SVG_PARAS_ICON: &str = "data:image/svg+xml,%3Csvg width='1080' height='1080' viewBox='0 0 1080 1080' fill='none' xmlns='http://www.w3.org/2000/svg'%3E%3Crect width='1080' height='1080' rx='10' fill='%230000BA'/%3E%3Cpath fill-rule='evenodd' clip-rule='evenodd' d='M335.238 896.881L240 184L642.381 255.288C659.486 259.781 675.323 263.392 689.906 266.718C744.744 279.224 781.843 287.684 801.905 323.725C827.302 369.032 840 424.795 840 491.014C840 557.55 827.302 613.471 801.905 658.779C776.508 704.087 723.333 726.74 642.381 726.74H468.095L501.429 896.881H335.238ZM387.619 331.329L604.777 369.407C614.008 371.807 622.555 373.736 630.426 375.513C660.02 382.193 680.042 386.712 690.869 405.963C704.575 430.164 711.428 459.95 711.428 495.321C711.428 530.861 704.575 560.731 690.869 584.932C677.163 609.133 648.466 621.234 604.777 621.234H505.578L445.798 616.481L387.619 331.329Z' fill='white'/%3E%3C/svg%3E";

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    NonFungibleToken,
    Metadata,
    TokenMetadata,
    Enumeration,
    Approval,
    // CUSTOM
    TokenSeriesById,
    TokensBySeriesInner { token_series: String },
    TokensPerOwner { account_hash: Vec<u8> },
    MarketDataTransactionFee,
    TokenMetadataAdmins,
    DefaultTokenMetadata
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new_default_meta(owner_id: ValidAccountId, treasury_id: ValidAccountId) -> Self {
        Self::new(
            owner_id,
            treasury_id,
            NFTContractMetadata {
                spec: NFT_METADATA_SPEC.to_string(),
                name: "Paras Collectibles".to_string(),
                symbol: "PARAS".to_string(),
                icon: Some(DATA_IMAGE_SVG_PARAS_ICON.to_string()),
                base_uri: Some("https://ipfs.fleek.co/ipfs".to_string()),
                reference: None,
                reference_hash: None,
            },
            500,
            TokenMetadata {
                title: Some("Default Title".to_string()), 
                description: Some("Default Description".to_string()),
                media: None,
                media_hash: None, 
                copies: None,
                issued_at: None,
                expires_at: None,
                starts_at: None,
                updated_at: None,
                extra: None,
                reference: None,
                reference_hash: None
            }
        )
    }

    #[init]
    pub fn new(
        owner_id: ValidAccountId, 
        treasury_id: ValidAccountId, 
        metadata: NFTContractMetadata,
        current_fee: u16,
        default_token_metadata: TokenMetadata,
    ) -> Self {
        metadata.assert_valid();
        let mut this = Self {
            tokens: NonFungibleToken::new(
                StorageKey::NonFungibleToken,
                owner_id.clone(),
                Some(StorageKey::TokenMetadata),
                Some(StorageKey::Enumeration),
                Some(StorageKey::Approval),
            ),
            token_series_by_id: UnorderedMap::new(StorageKey::TokenSeriesById),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
            treasury_id: treasury_id.to_string(),
            transaction_fee: TransactionFee {
                next_fee: None,
                start_time: None,
                current_fee
            },
            market_data_transaction_fee: MarketDataTransactionFee{
                transaction_fee: UnorderedMap::new(StorageKey::MarketDataTransactionFee)
            },
            token_metadata_admins: LookupSet::new(StorageKey::TokenMetadataAdmins),
            default_token_metadata: LazyOption::new(StorageKey::DefaultTokenMetadata, Some(&default_token_metadata)),
        };
        this.token_metadata_admins.insert(&owner_id.into());
        this
    }

    // #[payable]
    // pub fn set_transaction_fee(&mut self, next_fee: u16, start_time: Option<TimestampSec>) {
    //     assert!(false, "'set_transaction_fee' is forbidden");

    //     assert_one_yocto();
    //     assert_eq!(
    //         env::predecessor_account_id(),
    //         self.tokens.owner_id,
    //         "Paras: Owner only"
    //     );

    //     assert!(
    //         next_fee < 10_000,
    //         "Paras: transaction fee is more than 10_000"
    //     );

    //     if start_time.is_none() {
    //         self.transaction_fee.current_fee = next_fee;
    //         self.transaction_fee.next_fee = None;
    //         self.transaction_fee.start_time = None;
    //         return
    //     } else {
    //         let start_time: TimestampSec = start_time.unwrap();
    //         assert!(
    //             start_time > to_sec(env::block_timestamp()),
    //             "start_time is less than current block_timestamp"
    //         );
    //         self.transaction_fee.next_fee = Some(next_fee);
    //         self.transaction_fee.start_time = Some(start_time);
    //     }
    // }

    // pub fn calculate_market_data_transaction_fee(&mut self, token_series_id: &TokenSeriesId) -> u128{
    //     assert!(false, "'calculate_market_data_transaction_fee' is forbidden");
    //     if let Some(transaction_fee) = self.market_data_transaction_fee.transaction_fee.get(&token_series_id){
    //         return transaction_fee;
    //     }

    //     // fallback to default transaction fee
    //     self.calculate_current_transaction_fee()
    // }


    // pub fn calculate_current_transaction_fee(&mut self) -> u128 {
    //     assert!(false, "'calculate_current_transaction_fee' is forbidden");
    //     let transaction_fee: &TransactionFee = &self.transaction_fee;
    //     if transaction_fee.next_fee.is_some() {
    //         if to_sec(env::block_timestamp()) >= transaction_fee.start_time.unwrap() {
    //             self.transaction_fee.current_fee = transaction_fee.next_fee.unwrap();
    //             self.transaction_fee.next_fee = None;
    //             self.transaction_fee.start_time = None;
    //         }
    //     }
    //     self.transaction_fee.current_fee as u128
    // }

    pub fn get_transaction_fee(&self) -> &TransactionFee {
        &self.transaction_fee
    }

    pub fn get_market_data_transaction_fee (&self, token_series_id: &TokenId) -> u128{
        if let Some(transaction_fee) = self.market_data_transaction_fee.transaction_fee.get(&token_series_id){
            return transaction_fee;
        }
        // fallback to default transaction fee
        self.transaction_fee.current_fee as u128
    }


    // Treasury
    // #[payable]
    // pub fn set_treasury(&mut self, treasury_id: ValidAccountId) {
    //     assert_one_yocto();
    //     assert_eq!(
    //         env::predecessor_account_id(),
    //         self.tokens.owner_id,
    //         "Paras: Owner only"
    //     );
    //     self.treasury_id = treasury_id.to_string();
    // }

    // CUSTOM

    // #[payable]
    // pub fn nft_create_series(
    //     &mut self,
    //     creator_id: Option<ValidAccountId>,
    //     token_metadata: TokenMetadata,
    //     price: Option<U128>,
    //     royalty: Option<HashMap<AccountId, u32>>,
    // ) -> TokenSeriesJson {
    //     let initial_storage_usage = env::storage_usage();
    //     let caller_id = env::predecessor_account_id();

    //     assert!(false, "'nft_create_series' is forbidden, use 'nft_buy'");

    //     if creator_id.is_some() {
    //         assert_eq!(creator_id.unwrap().to_string(), caller_id, "Paras: Caller is not creator_id");
    //     }

    //     let token_series_id = format!("{}", (self.token_series_by_id.len() + 1));

    //     assert!(
    //         self.token_series_by_id.get(&token_series_id).is_none(),
    //         "Paras: duplicate token_series_id"
    //     );

    //     let title = token_metadata.title.clone();
    //     assert!(title.is_some(), "Paras: token_metadata.title is required");
        

    //     let mut total_perpetual = 0;
    //     let mut total_accounts = 0;
    //     let royalty_res: HashMap<AccountId, u32> = if let Some(royalty) = royalty {
    //         for (k , v) in royalty.iter() {
    //             if !is_valid_account_id(k.as_bytes()) {
    //                 env::panic("Not valid account_id for royalty".as_bytes());
    //             };
    //             total_perpetual += *v;
    //             total_accounts += 1;
    //         }
    //         royalty
    //     } else {
    //         HashMap::new()
    //     };

    //     assert!(total_accounts <= 50, "Paras: royalty exceeds 50 accounts");

    //     assert!(
    //         total_perpetual <= 9000,
    //         "Paras Exceeds maximum royalty -> 9000",
    //     );

    //     let price_res: Option<u128> = if price.is_some() {
    //         assert!(
    //             price.unwrap().0 < MAX_PRICE,
    //             "Paras: price higher than {}",
    //             MAX_PRICE
    //         );
    //         Some(price.unwrap().0)
    //     } else {
    //         None
    //     };

    //     self.token_series_by_id.insert(&token_series_id, &TokenSeries{
    //         metadata: token_metadata.clone(),
    //         creator_id: caller_id.to_string(),
    //         tokens: UnorderedSet::new(
    //             StorageKey::TokensBySeriesInner {
    //                 token_series: token_series_id.clone(),
    //             }
    //             .try_to_vec()
    //             .unwrap(),
    //         ),
    //         price: price_res,
    //         is_mintable: true,
    //         royalty: royalty_res.clone(),
    //     });

    //     // set market data transaction fee
    //     let current_transaction_fee = self.calculate_current_transaction_fee();
    //     self.market_data_transaction_fee.transaction_fee.insert(&token_series_id, &current_transaction_fee);

    //     env::log(
    //         json!({
    //             "type": "nft_create_series",
    //             "params": {
    //                 "token_series_id": token_series_id,
    //                 "token_metadata": token_metadata,
    //                 "creator_id": caller_id,
    //                 "price": price,
    //                 "royalty": royalty_res,
    //                 "transaction_fee": &current_transaction_fee.to_string()
    //             }
    //         })
    //         .to_string()
    //         .as_bytes(),
    //     );

    //     refund_deposit(env::storage_usage() - initial_storage_usage, 0);

		//     TokenSeriesJson{
    //         token_series_id,
		// 	      metadata: token_metadata,
		// 	      creator_id: caller_id.into(),
    //         royalty: royalty_res,
    //         transaction_fee: Some(current_transaction_fee.into()) 
		//     }

    // }

    #[payable]
    pub fn nft_buy(&mut self) -> TokenId {

        // let initial_storage_usage = env::storage_usage();
        let attached_deposit = env::attached_deposit();
        let receiver_id = env::predecessor_account_id();

        self.asset_max_supply_limit();
        
        assert!(
            attached_deposit == NFT_TOTAL_PRICE,
            "Attached deposit must be equal to : {}",
            NFT_TOTAL_PRICE
        );

        // Create Series
        let token_series_id = format!("{}", (self.token_series_by_id.len() + 1));
        assert!(self.token_series_by_id.get(&token_series_id).is_none(), "Paras: duplicate token_series_id");
        
        let royalty_res = HashMap::new();
        let token_metadata = self.default_token_metadata.get().expect("Default Token Metadata is not set");
        
        self.token_series_by_id.insert(&token_series_id, &TokenSeries{
            metadata: token_metadata.clone(),
            creator_id: receiver_id.to_string(),
            tokens: UnorderedSet::new(
                StorageKey::TokensBySeriesInner {
                    token_series: token_series_id.clone(),
                }
                .try_to_vec()
                .unwrap(),
            ),
            price: Some(Balance::from(NFT_PRICE)),
            is_mintable: true,
            royalty: royalty_res.clone(),
        });

        // set market data transaction fee
        // let current_transaction_fee = self.calculate_current_transaction_fee();
        // self.market_data_transaction_fee.transaction_fee.insert(&token_series_id, &current_transaction_fee);

        env::log(
            json!({
                "type": "nft_create_series",
                "params": {
                    "token_series_id": token_series_id,
                    "token_metadata": token_metadata,
                    "creator_id": receiver_id,
                    "price": Some(NFT_PRICE.to_string()),
                    "royalty": royalty_res,
                    "transaction_fee": NFT_REGISTRATION_FEE.to_string()
                }
            })
            .to_string()
            .as_bytes(),
        );

        // let token_series = self.token_series_by_id.get(&token_series_id).expect("Paras: Token series not exist");
        // let price: u128 = token_series.price.expect("Paras: not for sale");

        let token_id: TokenId = self._nft_mint_series(token_series_id.clone(), receiver_id.to_string());

        // let for_treasury = NFT_PRICE * self.calculate_market_data_transaction_fee(&token_series_id) / 10_000u128;
        // let price_deducted = NFT_PRICE - for_treasury;
        // Promise::new(token_series.creator_id).transfer(price_deducted);

        // if for_treasury != 0 {
        //     Promise::new(self.treasury_id.clone()).transfer(for_treasury);
        // }

        // refund_deposit(env::storage_usage() - initial_storage_usage, NFT_PRICE);

        Promise::new(self.treasury_id.clone()).transfer(NFT_PRICE);

        NearEvent::log_nft_mint(
            receiver_id.to_string(),
            vec![token_id.clone()],
            Some(json!({"price": NFT_PRICE.to_string()}).to_string())
        );

        token_id
    }

    
    // #[payable]
    // pub fn nft_mint(
    //     &mut self, 
    //     token_series_id: TokenSeriesId,
    //     receiver_id: ValidAccountId
    // ) -> TokenId {
        
    //     assert!(false, "'nft_mint' is forbidden, use 'nft_buy'");

    //     let initial_storage_usage = env::storage_usage();

    //     let token_series = self.token_series_by_id.get(&token_series_id).expect("Paras: Token series not exist");
    //     assert_eq!(env::predecessor_account_id(), token_series.creator_id, "Paras: not creator");
    //     let token_id: TokenId = self._nft_mint_series(token_series_id, receiver_id.to_string());

    //     refund_deposit(env::storage_usage() - initial_storage_usage, 0);

    //     NearEvent::log_nft_mint(
    //         receiver_id.to_string(),
    //         vec![token_id.clone()],
    //         None,
    //     );

    //     token_id
    // }

    // #[payable]
    // pub fn nft_mint_and_approve(
    //     &mut self, 
    //     token_series_id: TokenSeriesId, 
    //     account_id: ValidAccountId,
    //     msg: Option<String>,
    // ) -> Option<Promise> {
    //     assert!(false, "'nft_mint_and_approve' is forbidden, use 'nft_buy'");
    //     let initial_storage_usage = env::storage_usage();

    //     let token_series = self.token_series_by_id.get(&token_series_id).expect("Paras: Token series not exist");
    //     assert_eq!(env::predecessor_account_id(), token_series.creator_id, "Paras: not creator");
    //     let token_id: TokenId = self._nft_mint_series(token_series_id, token_series.creator_id.clone());

    //     // Need to copy the nft_approve code here to solve the gas problem
    //     // get contract-level LookupMap of token_id to approvals HashMap
    //     let approvals_by_id = self.tokens.approvals_by_id.as_mut().unwrap();

    //     // update HashMap of approvals for this token
    //     let approved_account_ids =
    //         &mut approvals_by_id.get(&token_id).unwrap_or_else(|| HashMap::new());
    //     let account_id: AccountId = account_id.into();
    //     let approval_id: u64 =
    //         self.tokens.next_approval_id_by_id.as_ref().unwrap().get(&token_id).unwrap_or_else(|| 1u64);
    //     approved_account_ids.insert(account_id.clone(), approval_id);

    //     // save updated approvals HashMap to contract's LookupMap
    //     approvals_by_id.insert(&token_id, &approved_account_ids);

    //     // increment next_approval_id for this token
    //     self.tokens.next_approval_id_by_id.as_mut().unwrap().insert(&token_id, &(approval_id + 1));

    //     refund_deposit(env::storage_usage() - initial_storage_usage, 0);

    //     NearEvent::log_nft_mint(
    //         token_series.creator_id.clone(),
    //         vec![token_id.clone()],
    //         None,
    //     );

    //     if let Some(msg) = msg {
    //         Some(ext_approval_receiver::nft_on_approve(
    //             token_id,
    //             token_series.creator_id,
    //             approval_id,
    //             msg,
    //             &account_id,
    //             NO_DEPOSIT,
    //             env::prepaid_gas() - GAS_FOR_NFT_APPROVE - GAS_FOR_MINT,
    //         ))
    //     } else {
    //         None
    //     }
    // }

    fn _nft_mint_series(
        &mut self, 
        token_series_id: TokenSeriesId,
        receiver_id: AccountId
    ) -> TokenId {
        let mut token_series = self.token_series_by_id.get(&token_series_id).expect("Paras: Token series not exist");
        assert!(
            token_series.is_mintable,
            "Paras: Token series is not mintable"
        );

        let num_tokens = token_series.tokens.len();
        let max_copies = token_series.metadata.copies.unwrap_or(u64::MAX);
        assert!(num_tokens < max_copies, "Series supply maxed");

        if (num_tokens + 1) >= max_copies {
            token_series.is_mintable = false;
            token_series.price = None;
        }

        let token_id = format!("{}{}{}", &token_series_id, TOKEN_DELIMETER, num_tokens + 1);
        token_series.tokens.insert(&token_id);
        self.token_series_by_id.insert(&token_series_id, &token_series);

        // you can add custom metadata to each token here
        let metadata = Some(TokenMetadata {
            title: None,          // ex. "Arch Nemesis: Mail Carrier" or "Parcel #5055"
            description: None,    // free-form description
            media: None, // URL to associated media, preferably to decentralized, content-addressed storage
            media_hash: None, // Base64-encoded sha256 hash of content referenced by the `media` field. Required if `media` is included.
            copies: None, // number of copies of this set of metadata in existence when token was minted.
            issued_at: Some(env::block_timestamp().to_string()), // ISO 8601 datetime when token was issued or minted
            expires_at: None, // ISO 8601 datetime when token expires
            starts_at: None, // ISO 8601 datetime when token starts being valid
            updated_at: None, // ISO 8601 datetime when token was last updated
            extra: None, // anything extra the NFT wants to store on-chain. Can be stringified JSON.
            reference: None, // URL to an off-chain JSON file with more info.
            reference_hash: None, // Base64-encoded sha256 hash of JSON from reference field. Required if `reference` is included.
        });

        //let token = self.tokens.mint(token_id, receiver_id, metadata);
        // From : https://github.com/near/near-sdk-rs/blob/master/near-contract-standards/src/non_fungible_token/core/core_impl.rs#L359
        // This allows lazy minting

        let owner_id: AccountId = receiver_id;
        self.tokens.owner_by_id.insert(&token_id, &owner_id);

        self.tokens
            .token_metadata_by_id
            .as_mut()
            .and_then(|by_id| by_id.insert(&token_id, &metadata.as_ref().unwrap()));

         if let Some(tokens_per_owner) = &mut self.tokens.tokens_per_owner {
             let mut token_ids = tokens_per_owner.get(&owner_id).unwrap_or_else(|| {
                 UnorderedSet::new(StorageKey::TokensPerOwner {
                     account_hash: env::sha256(&owner_id.as_bytes()),
                 })
             });
             token_ids.insert(&token_id);
             tokens_per_owner.insert(&owner_id, &token_ids);
         }


        token_id
    }



    // #[payable]
    // pub fn nft_decrease_series_copies(
    //     &mut self, 
    //     token_series_id: TokenSeriesId, 
    //     decrease_copies: U64
    // ) -> U64 {
    //     assert!(false, "'nft_decrease_series_copies' is forbidden'");

    //     assert_one_yocto();

    //     let mut token_series = self.token_series_by_id.get(&token_series_id).expect("Token series not exist");
    //     assert_eq!(
    //         env::predecessor_account_id(),
    //         token_series.creator_id,
    //         "Paras: Creator only"
    //     );

    //     let minted_copies = token_series.tokens.len();
    //     let copies = token_series.metadata.copies.unwrap();

    //     assert!(
    //         (copies - decrease_copies.0) >= minted_copies,
    //         "Paras: cannot decrease supply, already minted : {}", minted_copies
    //     );

    //     let is_non_mintable = if (copies - decrease_copies.0) == minted_copies {
    //         token_series.is_mintable = false;
    //         true
    //     } else {
    //         false
    //     };

    //     token_series.metadata.copies = Some(copies - decrease_copies.0);

    //     self.token_series_by_id.insert(&token_series_id, &token_series);
    //     env::log(
    //         json!({
    //             "type": "nft_decrease_series_copies",
    //             "params": {
    //                 "token_series_id": token_series_id,
    //                 "copies": U64::from(token_series.metadata.copies.unwrap()),
    //                 "is_non_mintable": is_non_mintable,
    //             }
    //         })
    //         .to_string()
    //         .as_bytes(),
    //     );
    //     U64::from(token_series.metadata.copies.unwrap())
    // }

    // #[payable]
    // pub fn nft_set_series_price(&mut self, token_series_id: TokenSeriesId, price: Option<U128>) -> Option<U128> {
    //     assert!(false, "'nft_set_series_price' is forbidden");
    //     assert_one_yocto();

    //     let mut token_series = self.token_series_by_id.get(&token_series_id).expect("Token series not exist");
    //     assert_eq!(
    //         env::predecessor_account_id(),
    //         token_series.creator_id,
    //         "Paras: Creator only"
    //     );

    //     assert_eq!(
    //         token_series.is_mintable,
    //         true,
    //         "Paras: token series is not mintable"
    //     );

    //     if price.is_none() {
    //         token_series.price = None;
    //     } else {
    //         assert!(
    //             price.unwrap().0 < MAX_PRICE,
    //             "Paras: price higher than {}",
    //             MAX_PRICE
    //         );
    //         token_series.price = Some(price.unwrap().0);
    //     }

    //     self.token_series_by_id.insert(&token_series_id, &token_series);

    //     // set market data transaction fee
    //     let current_transaction_fee = self.calculate_current_transaction_fee();
    //     self.market_data_transaction_fee.transaction_fee.insert(&token_series_id, &current_transaction_fee);

    //     env::log(
    //         json!({
    //             "type": "nft_set_series_price",
    //             "params": {
    //                 "token_series_id": token_series_id,
    //                 "price": price,
    //                 "transaction_fee": current_transaction_fee.to_string()
    //             }
    //         })
    //         .to_string()
    //         .as_bytes(),
    //     );
    //     return price;
    // }

    // #[payable]
    // pub fn nft_burn(&mut self, token_id: TokenId) {
    //     assert!(false, "'nft_burn' is forbidden");
    //     assert_one_yocto();

    //     let owner_id = self.tokens.owner_by_id.get(&token_id).unwrap();
    //     assert_eq!(
    //         owner_id,
    //         env::predecessor_account_id(),
    //         "Token owner only"
    //     );

    //     if let Some(next_approval_id_by_id) = &mut self.tokens.next_approval_id_by_id {
    //         next_approval_id_by_id.remove(&token_id);
    //     }

    //     if let Some(approvals_by_id) = &mut self.tokens.approvals_by_id {
    //         approvals_by_id.remove(&token_id);
    //     }

    //     if let Some(tokens_per_owner) = &mut self.tokens.tokens_per_owner {
    //         let mut token_ids = tokens_per_owner.get(&owner_id).unwrap();
    //         token_ids.remove(&token_id);
    //         tokens_per_owner.insert(&owner_id, &token_ids);
    //     }

    //     if let Some(token_metadata_by_id) = &mut self.tokens.token_metadata_by_id {
    //         token_metadata_by_id.remove(&token_id);
    //     }

    //     self.tokens.owner_by_id.remove(&token_id);

    //     NearEvent::log_nft_burn(
    //         owner_id,
    //         vec![token_id],
    //         None,
    //         None,
    //     );
    // }

    // CUSTOM VIEWS

	pub fn nft_get_series_single(&self, token_series_id: TokenSeriesId) -> TokenSeriesJson {
		let token_series = self.token_series_by_id.get(&token_series_id).expect("Series does not exist");
        let current_transaction_fee = self.get_market_data_transaction_fee(&token_series_id);
		TokenSeriesJson{
            token_series_id,
			metadata: token_series.metadata,
			creator_id: token_series.creator_id,
            royalty: token_series.royalty,
            transaction_fee: Some(current_transaction_fee.into()) 
		}
	}

    pub fn nft_get_series_format(self) -> (char, &'static str, &'static str) {
        (TOKEN_DELIMETER, TITLE_DELIMETER, EDITION_DELIMETER)
    }

    pub fn nft_get_series_price(self, token_series_id: TokenSeriesId) -> Option<U128> {
        let price = self.token_series_by_id.get(&token_series_id).unwrap().price;
        match price {
            Some(p) => return Some(U128::from(p)),
            None => return None
        };
    }

    pub fn nft_get_series(
        &self,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<TokenSeriesJson> {
        let start_index: u128 = from_index.map(From::from).unwrap_or_default();
        assert!(
            (self.token_series_by_id.len() as u128) > start_index,
            "Out of bounds, please use a smaller from_index."
        );
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");

        self.token_series_by_id
            .iter()
            .skip(start_index as usize)
            .take(limit)
            .map(|(token_series_id, token_series)| TokenSeriesJson{
                token_series_id,
                metadata: token_series.metadata,
                creator_id: token_series.creator_id,
                royalty: token_series.royalty,
                transaction_fee: None 
            })
            .collect()
    }

    pub fn nft_supply_for_series(&self, token_series_id: TokenSeriesId) -> U64 {
        self.token_series_by_id.get(&token_series_id).expect("Token series not exist").tokens.len().into()
    }

    pub fn nft_tokens_by_series(
        &self,
        token_series_id: TokenSeriesId,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<Token> {
        let start_index: u128 = from_index.map(From::from).unwrap_or_default();
        let tokens = self.token_series_by_id.get(&token_series_id).unwrap().tokens;
        assert!(
            (tokens.len() as u128) > start_index,
            "Out of bounds, please use a smaller from_index."
        );
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");

        tokens
            .iter()
            .skip(start_index as usize)
            .take(limit)
            .map(|token_id| self.nft_token(token_id).unwrap())
            .collect()
    }

    pub fn nft_token(&self, token_id: TokenId) -> Option<Token> {
        let owner_id = self.tokens.owner_by_id.get(&token_id)?;
        let approved_account_ids = self
            .tokens
            .approvals_by_id
            .as_ref()
            .and_then(|by_id| by_id.get(&token_id).or_else(|| Some(HashMap::new())));

        // CUSTOM (switch metadata for the token_series metadata)
        let mut token_id_iter = token_id.split(TOKEN_DELIMETER);
        let token_series_id = token_id_iter.next().unwrap().parse().unwrap();
        let series_metadata = self.token_series_by_id.get(&token_series_id).unwrap().metadata;

        let mut token_metadata = self.tokens.token_metadata_by_id.as_ref().unwrap().get(&token_id).unwrap();

        token_metadata.title = series_metadata.title;
        token_metadata.reference = series_metadata.reference;
        token_metadata.media = series_metadata.media;
        token_metadata.copies = series_metadata.copies;
        token_metadata.extra = series_metadata.extra;

        Some(Token {
            token_id,
            owner_id,
            metadata: Some(token_metadata),
            approved_account_ids,
        })
    }

    #[payable]
    pub fn nft_transfer(
        &mut self,
        receiver_id: ValidAccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
    ) {
        let sender_id = env::predecessor_account_id();
        let previous_owner_id = self.tokens.owner_by_id.get(&token_id).expect("Token not found");
        let receiver_id_str = receiver_id.to_string();
        self.tokens.nft_transfer(receiver_id, token_id.clone(), approval_id, memo.clone());

        let authorized_id : Option<AccountId> = if sender_id != previous_owner_id {
            Some(sender_id)
        } else {
             None
        };

        NearEvent::log_nft_transfer(
            previous_owner_id,
            receiver_id_str,
            vec![token_id],
            memo,
             authorized_id,
        );
    }

    #[payable]
    pub fn nft_transfer_call(
        &mut self,
        receiver_id: ValidAccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<bool> {
        assert_one_yocto();
        let sender_id = env::predecessor_account_id();
        let (previous_owner_id, old_approvals) = self.tokens.internal_transfer(
            &sender_id,
            receiver_id.as_ref(),
            &token_id,
            approval_id,
            memo.clone(),
        );

        let authorized_id : Option<AccountId> = if sender_id != previous_owner_id {
            Some(sender_id.clone())
        } else {
            None
        };

        NearEvent::log_nft_transfer(
            previous_owner_id.clone(),
            receiver_id.to_string(),
            vec![token_id.clone()],
            memo,
            authorized_id,
        );

        // Initiating receiver's call and the callback
        ext_non_fungible_token_receiver::nft_on_transfer(
            sender_id,
            previous_owner_id.clone(),
            token_id.clone(),
            msg,
            receiver_id.as_ref(),
            NO_DEPOSIT,
            env::prepaid_gas() - GAS_FOR_NFT_TRANSFER_CALL,
        )
        .then(ext_self::nft_resolve_transfer(
            previous_owner_id,
            receiver_id.into(),
            token_id,
            old_approvals,
            &env::current_account_id(),
            NO_DEPOSIT,
            GAS_FOR_RESOLVE_TRANSFER,
        ))
        .into()
    }

    // CUSTOM enumeration standard modified here because no macro below

    pub fn nft_total_supply(&self) -> U128 {
        (self.tokens.owner_by_id.len() as u128).into()
    }

    pub fn nft_tokens(&self, from_index: Option<U128>, limit: Option<u64>) -> Vec<Token> {
        // Get starting index, whether or not it was explicitly given.
        // Defaults to 0 based on the spec:
        // https://nomicon.io/Standards/NonFungibleToken/Enumeration.html#interface
        let start_index: u128 = from_index.map(From::from).unwrap_or_default();
        assert!(
            (self.tokens.owner_by_id.len() as u128) > start_index,
            "Out of bounds, please use a smaller from_index."
        );
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
        self.tokens
            .owner_by_id
            .iter()
            .skip(start_index as usize)
            .take(limit)
            .map(|(token_id, _)| self.nft_token(token_id).unwrap())
            .collect()
    }

    pub fn nft_supply_for_owner(self, account_id: ValidAccountId) -> U128 {
        let tokens_per_owner = self.tokens.tokens_per_owner.expect(
            "Could not find tokens_per_owner when calling a method on the enumeration standard.",
        );
        tokens_per_owner
            .get(account_id.as_ref())
            .map(|account_tokens| U128::from(account_tokens.len() as u128))
            .unwrap_or(U128(0))
    }

    pub fn nft_tokens_for_owner(
        &self,
        account_id: ValidAccountId,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<Token> {
        let tokens_per_owner = self.tokens.tokens_per_owner.as_ref().expect(
            "Could not find tokens_per_owner when calling a method on the enumeration standard.",
        );
        let token_set = if let Some(token_set) = tokens_per_owner.get(account_id.as_ref()) {
            token_set
        } else {
            return vec![];
        };
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
        let start_index: u128 = from_index.map(From::from).unwrap_or_default();
        assert!(
            token_set.len() as u128 > start_index,
            "Out of bounds, please use a smaller from_index."
        );
        token_set
            .iter()
            .skip(start_index as usize)
            .take(limit)
            .map(|token_id| self.nft_token(token_id).unwrap())
            .collect()
    }

    pub fn nft_payout(
        &self, 
        token_id: TokenId,
        balance: U128, 
        max_len_payout: u32
    ) -> Payout{
        let owner_id = self.tokens.owner_by_id.get(&token_id).expect("No token id");
        let mut token_id_iter = token_id.split(TOKEN_DELIMETER);
        let token_series_id = token_id_iter.next().unwrap().parse().unwrap();
        let royalty = self.token_series_by_id.get(&token_series_id).expect("no type").royalty;

        assert!(royalty.len() as u32 <= max_len_payout, "Market cannot payout to that many receivers");

        let balance_u128: u128 = balance.into();

        let mut payout: Payout = Payout { payout: HashMap::new() };
        let mut total_perpetual = 0;

        for (k, v) in royalty.iter() {
            if *k != owner_id {
                let key = k.clone();
                payout.payout.insert(key, royalty_to_payout(*v, balance_u128));
                total_perpetual += *v;
            }
        }
        payout.payout.insert(owner_id, royalty_to_payout(10000 - total_perpetual, balance_u128));
        payout
    }

    #[payable]
    pub fn nft_transfer_payout(
        &mut self, 
        receiver_id: ValidAccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        balance: Option<U128>,
        max_len_payout: Option<u32>
    ) -> Option<Payout> {
        assert_one_yocto();

        let sender_id = env::predecessor_account_id();
        // Transfer
        let previous_token = self.nft_token(token_id.clone()).expect("no token");
        self.tokens.nft_transfer(receiver_id.clone(), token_id.clone(), approval_id, None);

        // Payout calculation
        let previous_owner_id = previous_token.owner_id;
        let mut total_perpetual = 0;
        let payout = if let Some(balance) = balance {
            let balance_u128: u128 = u128::from(balance);
            let mut payout: Payout = Payout { payout: HashMap::new() };

            let mut token_id_iter = token_id.split(TOKEN_DELIMETER);
            let token_series_id = token_id_iter.next().unwrap().parse().unwrap();
            let royalty = self.token_series_by_id.get(&token_series_id).expect("no type").royalty;

            assert!(royalty.len() as u32 <= max_len_payout.unwrap(), "Market cannot payout to that many receivers");
            for (k, v) in royalty.iter() {
                let key = k.clone();
                if key != previous_owner_id {
                    payout.payout.insert(key, royalty_to_payout(*v, balance_u128));
                    total_perpetual += *v;
                }
            }

            assert!(
                total_perpetual <= 10000,
                "Total payout overflow"
            );

            payout.payout.insert(previous_owner_id.clone(), royalty_to_payout(10000 - total_perpetual, balance_u128));
            Some(payout)
        } else {
            None
        };

        let authorized_id : Option<AccountId> = if sender_id != previous_owner_id {
            Some(sender_id)
        } else {
            None
        };

        NearEvent::log_nft_transfer(
            previous_owner_id,
            receiver_id.to_string(),
            vec![token_id],
            None,
            authorized_id,
        );

        payout
    }

    pub fn get_owner(&self) -> AccountId {
        self.tokens.owner_id.clone()
    }


    #[payable]
    pub fn nft_set_metadata(
        &mut self,
        token_id: TokenId,
        token_metadata: TokenMetadata
    ) {
        self.assert_token_metadata_admin();
        let mut token_id_iter = token_id.split(TOKEN_DELIMETER);
        let token_series_id = token_id_iter.next().unwrap().parse().unwrap();

        if self.tokens.owner_by_id.get(&token_id).is_none() {
            env::panic("Token id does not exist".as_bytes());
        };

        if let Some(token_series_by_id) = &mut self.token_series_by_id.get(&token_series_id) {
            token_series_by_id.metadata = token_metadata.clone();
            self.token_series_by_id.insert(&token_series_id, token_series_by_id);

        } else {
            env::panic("Token Metadata series is not found".as_bytes());
        };

        if let Some(token_metadata_by_id) = &mut self.tokens.token_metadata_by_id {
            token_metadata_by_id.insert(&token_id, &token_metadata);
        } else {
            env::panic("Token Metadata extension is not set".as_bytes());
        };
        
    }

    #[payable]
    pub fn set_default_token_metadata(
        &mut self,
        default_token_metadata: TokenMetadata
    ) {
        self.assert_token_metadata_admin();
        self.default_token_metadata.set(&default_token_metadata);
    }

    fn assert_owner(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.tokens.owner_id,
            "Paras: Owner only"
        );
    }

    fn asset_max_supply_limit(&self) {
        let current_supply = self.token_series_by_id.len() as u128;
        assert!(current_supply < NFT_MAX_SUPPLY,
            "Max NFT supply is reached"
        );
    }

    fn assert_token_metadata_admin(&self) {
        assert!(self.token_metadata_admins.contains(&env::predecessor_account_id()),
            "This operation is restricted to token token metadata admin"
        );
    }

    pub fn add_token_metadata_admin(&mut self, account_id: ValidAccountId) {
        self.assert_owner();
        if !self.token_metadata_admins.insert(&account_id.into()) {
            env::panic("The account is already registered as a token metadata admin".as_bytes());
        }
    }

    pub fn remove_token_metadata_admin(&mut self, account_id: ValidAccountId) {
        self.assert_owner();
        if !self.token_metadata_admins.remove(&account_id.into()) {
            env::panic("The account is not registered as a token metadata admin".as_bytes());
        }
    }

}

fn royalty_to_payout(a: u32, b: Balance) -> U128 {
    U128(a as u128 * b / 10_000u128)
}

// near_contract_standards::impl_non_fungible_token_core!(Contract, tokens);
// near_contract_standards::impl_non_fungible_token_enumeration!(Contract, tokens);
near_contract_standards::impl_non_fungible_token_approval!(Contract, tokens);

#[near_bindgen]
impl NonFungibleTokenMetadataProvider for Contract {
    fn nft_metadata(&self) -> NFTContractMetadata {
        self.metadata.get().unwrap()
    }
}

#[near_bindgen]
impl NonFungibleTokenResolver for Contract {
    #[private]
    fn nft_resolve_transfer(
        &mut self,
        previous_owner_id: AccountId,
        receiver_id: AccountId,
        token_id: TokenId,
        approved_account_ids: Option<HashMap<AccountId, u64>>,
    ) -> bool {
        let resp: bool = self.tokens.nft_resolve_transfer(
            previous_owner_id.clone(),
            receiver_id.clone(),
            token_id.clone(),
            approved_account_ids,
        );

        // if not successful, return nft back to original owner
        if !resp {
            NearEvent::log_nft_transfer(
                receiver_id,
                previous_owner_id,
                vec![token_id],
                None,
                None,
            );
        }

        resp
    }
}

// from https://github.com/near/near-sdk-rs/blob/e4abb739ff953b06d718037aa1b8ab768db17348/near-contract-standards/src/non_fungible_token/utils.rs#L29

// fn refund_deposit(storage_used: u64, extra_spend: Balance) {
//     let required_cost = env::storage_byte_cost() * Balance::from(storage_used);
//     let attached_deposit = env::attached_deposit() - extra_spend;

//     assert!(
//         required_cost <= attached_deposit,
//         "Must attach {} yoctoNEAR to cover storage",
//         required_cost,
//     );

//     let refund = attached_deposit - required_cost;
//     if refund > 1 {
//         Promise::new(env::predecessor_account_id()).transfer(refund);
//     }
// }

// fn to_sec(timestamp: Timestamp) -> TimestampSec {
//     (timestamp / 10u64.pow(9)) as u32
// }

// #[cfg(all(test, not(target_arch = "wasm32")))]
// mod tests {
//     use super::*;
//     use near_sdk::test_utils::{accounts, VMContextBuilder};
//     use near_sdk::MockedBlockchain;
//     use near_sdk::{testing_env};

//     const STORAGE_FOR_CREATE_SERIES: Balance = 8540000000000000000000;
//     const STORAGE_FOR_MINT: Balance = 11280000000000000000000;

//     fn get_context(predecessor_account_id: ValidAccountId) -> VMContextBuilder {
//         let mut builder = VMContextBuilder::new();
//         builder
//             .current_account_id(accounts(0))
//             .signer_account_id(predecessor_account_id.clone())
//             .predecessor_account_id(predecessor_account_id);
//         builder
//     }

//     fn setup_contract() -> (VMContextBuilder, Contract) {
//         let mut context = VMContextBuilder::new();
//         testing_env!(context.predecessor_account_id(accounts(0)).build());
//         let contract = Contract::new_default_meta(accounts(0), accounts(4));
//         (context, contract)
//     }

//     #[test]
//     fn test_new() {
//         let mut context = get_context(accounts(1));
//         testing_env!(context.build());
//         let contract = Contract::new(
//             accounts(1),
//             accounts(4),
//             NFTContractMetadata {
//                 spec: NFT_METADATA_SPEC.to_string(),
//                 name: "Triple Triad".to_string(),
//                 symbol: "TRIAD".to_string(),
//                 icon: Some(DATA_IMAGE_SVG_PARAS_ICON.to_string()),
//                 base_uri: Some("https://ipfs.fleek.co/ipfs/".to_string()),
//                 reference: None,
//                 reference_hash: None,
//             },
//             500
//         );
//         testing_env!(context.is_view(true).build());
//         assert_eq!(contract.get_owner(), accounts(1).to_string());
//         assert_eq!(contract.nft_metadata().base_uri.unwrap(), "https://ipfs.fleek.co/ipfs/".to_string());
//         assert_eq!(contract.nft_metadata().icon.unwrap(), DATA_IMAGE_SVG_PARAS_ICON.to_string());
//     }

//     fn create_series(
//         contract: &mut Contract,
//         royalty: &HashMap<AccountId, u32>,
//         price: Option<U128>,
//         copies: Option<u64>,
//     ) {
//         contract.nft_create_series(
//             None,
//             TokenMetadata {
//                 title: Some("Tsundere land".to_string()),
//                 description: None,
//                 media: Some(
//                     "bafybeidzcan4nzcz7sczs4yzyxly4galgygnbjewipj6haco4kffoqpkiy".to_string()
//                 ),
//                 media_hash: None,
//                 copies: copies,
//                 issued_at: None,
//                 expires_at: None,
//                 starts_at: None,
//                 updated_at: None,
//                 extra: None,
//                 reference: Some(
//                     "bafybeicg4ss7qh5odijfn2eogizuxkrdh3zlv4eftcmgnljwu7dm64uwji".to_string()
//                 ),
//                 reference_hash: None,
//             },
//             price,
//             Some(royalty.clone()),
//         );
//     }

//     #[test]
//     fn test_create_series() {
//         let (mut context, mut contract) = setup_contract();
//         testing_env!(context
//             .predecessor_account_id(accounts(1))
//             .attached_deposit(STORAGE_FOR_CREATE_SERIES)
//             .build()
//         );

//         let mut royalty: HashMap<AccountId, u32> = HashMap::new();
//         royalty.insert(accounts(1).to_string(), 1000);
//         create_series(
//             &mut contract,
//             &royalty,
//             Some(U128::from(1 * 10u128.pow(24))),
//             None
//         );

//         let nft_series_return = contract.nft_get_series_single("1".to_string());
//         assert_eq!(
//             nft_series_return.creator_id,
//             accounts(1).to_string()
//         );

//         assert_eq!(
//             nft_series_return.token_series_id,
//             "1",
//         );

//         assert_eq!(
//             nft_series_return.royalty,
//             royalty,
//         );

//         assert!(
//             nft_series_return.metadata.copies.is_none()
//         );

//         assert_eq!(
//             nft_series_return.metadata.title.unwrap(),
//             "Tsundere land".to_string()
//         );

//         assert_eq!(
//             nft_series_return.metadata.reference.unwrap(),
//             "bafybeicg4ss7qh5odijfn2eogizuxkrdh3zlv4eftcmgnljwu7dm64uwji".to_string()
//         );

//     }

//     #[test]
//     fn test_buy() {
//         let (mut context, mut contract) = setup_contract();
//         testing_env!(context
//             .predecessor_account_id(accounts(1))
//             .attached_deposit(STORAGE_FOR_CREATE_SERIES)
//             .build()
//         );

//         let mut royalty: HashMap<AccountId, u32> = HashMap::new();
//         royalty.insert(accounts(1).to_string(), 1000);

//         create_series(
//             &mut contract,
//             &royalty,
//             Some(U128::from(1 * 10u128.pow(24))),
//             None
//         );

//         testing_env!(context
//             .predecessor_account_id(accounts(2))
//             .attached_deposit(1 * 10u128.pow(24) + STORAGE_FOR_MINT)
//             .build()
//         );

//         let token_id = contract.nft_buy();

//         let token_from_nft_token = contract.nft_token(token_id);
//         assert_eq!(
//             token_from_nft_token.unwrap().owner_id,
//             accounts(2).to_string()
//         )
//     }

//     #[test]
//     fn test_mint() {
//         let (mut context, mut contract) = setup_contract();
//         testing_env!(context
//             .predecessor_account_id(accounts(1))
//             .attached_deposit(STORAGE_FOR_CREATE_SERIES)
//             .build()
//         );

//         let mut royalty: HashMap<AccountId, u32> = HashMap::new();
//         royalty.insert(accounts(1).to_string(), 1000);

//         create_series(&mut contract, &royalty, None, None);

//         testing_env!(context
//             .predecessor_account_id(accounts(1))
//             .attached_deposit(STORAGE_FOR_MINT)
//             .build()
//         );

//         let token_id = contract.nft_mint("1".to_string(), accounts(2));

//         let token_from_nft_token = contract.nft_token(token_id);
//         assert_eq!(
//             token_from_nft_token.unwrap().owner_id,
//             accounts(2).to_string()
//         )
//     }

//     #[test]
//     #[should_panic(expected = "Paras: Token series is not mintable")]
//     fn test_invalid_mint_above_copies() {
//         let (mut context, mut contract) = setup_contract();
//         testing_env!(context
//             .predecessor_account_id(accounts(1))
//             .attached_deposit(STORAGE_FOR_CREATE_SERIES)
//             .build()
//         );

//         let mut royalty: HashMap<AccountId, u32> = HashMap::new();
//         royalty.insert(accounts(1).to_string(), 1000);

//         create_series(&mut contract, &royalty, None, Some(1));

//         testing_env!(context
//             .predecessor_account_id(accounts(1))
//             .attached_deposit(STORAGE_FOR_MINT)
//             .build()
//         );

//         contract.nft_mint("1".to_string(), accounts(2));
//         contract.nft_mint("1".to_string(), accounts(2));
//     }

//     #[test]
//     fn test_decrease_copies() {
//         let (mut context, mut contract) = setup_contract();
//         testing_env!(context
//             .predecessor_account_id(accounts(1))
//             .attached_deposit(STORAGE_FOR_CREATE_SERIES)
//             .build()
//         );

//         let mut royalty: HashMap<AccountId, u32> = HashMap::new();
//         royalty.insert(accounts(1).to_string(), 1000);

//         create_series(&mut contract, &royalty, None, Some(5));

//         testing_env!(context
//             .predecessor_account_id(accounts(1))
//             .attached_deposit(STORAGE_FOR_MINT)
//             .build()
//         );

//         contract.nft_mint("1".to_string(), accounts(2));
//         contract.nft_mint("1".to_string(), accounts(2));

//         testing_env!(context
//             .predecessor_account_id(accounts(1))
//             .attached_deposit(1)
//             .build()
//         );

//         contract.nft_decrease_series_copies("1".to_string(), U64::from(3));
//     }

//     #[test]
//     #[should_panic(expected = "Paras: cannot decrease supply, already minted : 2")]
//     fn test_invalid_decrease_copies() {
//         let (mut context, mut contract) = setup_contract();
//         testing_env!(context
//             .predecessor_account_id(accounts(1))
//             .attached_deposit(STORAGE_FOR_CREATE_SERIES)
//             .build()
//         );

//         let mut royalty: HashMap<AccountId, u32> = HashMap::new();
//         royalty.insert(accounts(1).to_string(), 1000);

//         create_series(&mut contract, &royalty, None, Some(5));

//         testing_env!(context
//             .predecessor_account_id(accounts(1))
//             .attached_deposit(STORAGE_FOR_MINT)
//             .build()
//         );

//         contract.nft_mint("1".to_string(), accounts(2));
//         contract.nft_mint("1".to_string(), accounts(2));

//         testing_env!(context
//             .predecessor_account_id(accounts(1))
//             .attached_deposit(1)
//             .build()
//         );

//         contract.nft_decrease_series_copies("1".to_string(), U64::from(4));
//     }

//     #[test]
//     #[should_panic( expected = "Paras: not for sale" )]
//     fn test_invalid_buy_price_null() {
//         let (mut context, mut contract) = setup_contract();
//         testing_env!(context
//             .predecessor_account_id(accounts(1))
//             .attached_deposit(STORAGE_FOR_CREATE_SERIES)
//             .build()
//         );

//         let mut royalty: HashMap<AccountId, u32> = HashMap::new();
//         royalty.insert(accounts(1).to_string(), 1000);

//         create_series(&mut contract, &royalty, Some(U128::from(1 * 10u128.pow(24))), None);

//         testing_env!(context
//             .predecessor_account_id(accounts(1))
//             .attached_deposit(1)
//             .build()
//         );

//         contract.nft_set_series_price("1".to_string(), None);

//         testing_env!(context
//             .predecessor_account_id(accounts(2))
//             .attached_deposit(1 * 10u128.pow(24) + STORAGE_FOR_MINT)
//             .build()
//         );

//         let token_id = contract.nft_buy();

//         let token_from_nft_token = contract.nft_token(token_id);
//         assert_eq!(
//             token_from_nft_token.unwrap().owner_id,
//             accounts(2).to_string()
//         )
//     }

//     #[test]
//     #[should_panic( expected = "Paras: price higher than 1000000000000000000000000000000000" )]
//     fn test_invalid_price_shouldnt_be_higher_than_max_price() {
//         let (mut context, mut contract) = setup_contract();
//         testing_env!(context
//             .predecessor_account_id(accounts(1))
//             .attached_deposit(STORAGE_FOR_CREATE_SERIES)
//             .build()
//         );

//         let mut royalty: HashMap<AccountId, u32> = HashMap::new();
//         royalty.insert(accounts(1).to_string(), 1000);

//         create_series(&mut contract, &royalty, Some(U128::from(1_000_000_000 * 10u128.pow(24))), None);

//         testing_env!(context
//             .predecessor_account_id(accounts(1))
//             .attached_deposit(1)
//             .build()
//         );
//     }

//     #[test]
//     fn test_nft_burn() {
//         let (mut context, mut contract) = setup_contract();
//         testing_env!(context
//             .predecessor_account_id(accounts(1))
//             .attached_deposit(STORAGE_FOR_CREATE_SERIES)
//             .build()
//         );

//         let mut royalty: HashMap<AccountId, u32> = HashMap::new();
//         royalty.insert(accounts(1).to_string(), 1000);

//         create_series(&mut contract, &royalty, None, None);

//         testing_env!(context
//             .predecessor_account_id(accounts(1))
//             .attached_deposit(STORAGE_FOR_MINT)
//             .build()
//         );

//         let token_id = contract.nft_mint("1".to_string(), accounts(2));

//         testing_env!(context
//             .predecessor_account_id(accounts(2))
//             .attached_deposit(1)
//             .build()
//         );

//         contract.nft_burn(token_id.clone());
//         let token = contract.nft_token(token_id);
//         assert!(token.is_none());
//     }

//     #[test]
//     fn test_nft_transfer() {
//         let (mut context, mut contract) = setup_contract();
//         testing_env!(context
//             .predecessor_account_id(accounts(1))
//             .attached_deposit(STORAGE_FOR_CREATE_SERIES)
//             .build()
//         );

//         let mut royalty: HashMap<AccountId, u32> = HashMap::new();
//         royalty.insert(accounts(1).to_string(), 1000);

//         create_series(&mut contract, &royalty, None, None);

//         testing_env!(context
//             .predecessor_account_id(accounts(1))
//             .attached_deposit(STORAGE_FOR_MINT)
//             .build()
//         );

//         let token_id = contract.nft_mint("1".to_string(), accounts(2));

//         testing_env!(context
//             .predecessor_account_id(accounts(2))
//             .attached_deposit(1)
//             .build()
//         );

//         contract.nft_transfer(accounts(3), token_id.clone(), None, None);

//         let token = contract.nft_token(token_id).unwrap();
//         assert_eq!(
//             token.owner_id,
//             accounts(3).to_string()
//         )
//     }

//     #[test]
//     fn test_nft_transfer_payout() {
//         let (mut context, mut contract) = setup_contract();
//         testing_env!(context
//             .predecessor_account_id(accounts(1))
//             .attached_deposit(STORAGE_FOR_CREATE_SERIES)
//             .build()
//         );

//         let mut royalty: HashMap<AccountId, u32> = HashMap::new();
//         royalty.insert(accounts(1).to_string(), 1000);

//         create_series(&mut contract, &royalty, None, None);

//         testing_env!(context
//             .predecessor_account_id(accounts(1))
//             .attached_deposit(STORAGE_FOR_MINT)
//             .build()
//         );

//         let token_id = contract.nft_mint("1".to_string(), accounts(2));

//         testing_env!(context
//             .predecessor_account_id(accounts(2))
//             .attached_deposit(1)
//             .build()
//         );

//         let payout = contract.nft_transfer_payout(
//             accounts(3),
//             token_id.clone(),
//             Some(0) ,
//             Some(U128::from(1 * 10u128.pow(24))),
//             Some(10)
//         );

//         let mut payout_calc: HashMap<AccountId, U128> = HashMap::new();
//         payout_calc.insert(
//             accounts(1).to_string(),
//             U128::from((1000 * (1 * 10u128.pow(24)))/10_000)
//         );
//         payout_calc.insert(
//             accounts(2).to_string(),
//             U128::from((9000 * (1 * 10u128.pow(24))) / 10_000)
//         );

//         assert_eq!(payout.unwrap().payout, payout_calc);

//         let token = contract.nft_token(token_id).unwrap();
//         assert_eq!(
//             token.owner_id,
//             accounts(3).to_string()
//         )
//     }

//     #[test]
//     fn test_change_transaction_fee_immediately() {
//         let (mut context, mut contract) = setup_contract();

//         testing_env!(context
//             .predecessor_account_id(accounts(0))
//             .attached_deposit(1)
//             .build()
//         );

//         contract.set_transaction_fee(100, None);

//         assert_eq!(contract.get_transaction_fee().current_fee, 100);
//     }

//     #[test]
//     fn test_change_transaction_fee_with_time() {
//         let (mut context, mut contract) = setup_contract();

//         testing_env!(context
//             .predecessor_account_id(accounts(0))
//             .attached_deposit(1)
//             .build()
//         );

//         assert_eq!(contract.get_transaction_fee().current_fee, 500);
//         assert_eq!(contract.get_transaction_fee().next_fee, None);
//         assert_eq!(contract.get_transaction_fee().start_time, None);

//         let next_fee: u16 = 100;
//         let start_time: Timestamp = 1618109122863866400;
//         let start_time_sec: TimestampSec = to_sec(start_time);
//         contract.set_transaction_fee(next_fee, Some(start_time_sec));

//         assert_eq!(contract.get_transaction_fee().current_fee, 500);
//         assert_eq!(contract.get_transaction_fee().next_fee, Some(next_fee));
//         assert_eq!(contract.get_transaction_fee().start_time, Some(start_time_sec));

//         testing_env!(context
//             .predecessor_account_id(accounts(1))
//             .block_timestamp(start_time + 1)
//             .build()
//         );

//         contract.calculate_current_transaction_fee();
//         assert_eq!(contract.get_transaction_fee().current_fee, next_fee);
//         assert_eq!(contract.get_transaction_fee().next_fee, None);
//         assert_eq!(contract.get_transaction_fee().start_time, None);
//     }

//     #[test]
//     fn test_transaction_fee_locked() {
//         let (mut context, mut contract) = setup_contract();

//         testing_env!(context
//             .predecessor_account_id(accounts(0))
//             .attached_deposit(1)
//             .build()
//         );

//         assert_eq!(contract.get_transaction_fee().current_fee, 500);
//         assert_eq!(contract.get_transaction_fee().next_fee, None);
//         assert_eq!(contract.get_transaction_fee().start_time, None);

//         let next_fee: u16 = 100;
//         let start_time: Timestamp = 1618109122863866400;
//         let start_time_sec: TimestampSec = to_sec(start_time);
//         contract.set_transaction_fee(next_fee, Some(start_time_sec));

//         let mut royalty: HashMap<AccountId, u32> = HashMap::new();
//         royalty.insert(accounts(1).to_string(), 1000);

//         testing_env!(context
//             .predecessor_account_id(accounts(0))
//             .attached_deposit(STORAGE_FOR_CREATE_SERIES)
//             .build()
//         );

//         create_series(&mut contract, &royalty, Some(U128::from(1 * 10u128.pow(24))), None);

//         testing_env!(context
//             .predecessor_account_id(accounts(0))
//             .attached_deposit(1)
//             .build()
//         );

//         contract.nft_set_series_price("1".to_string(), None);

//         assert_eq!(contract.get_transaction_fee().current_fee, 500);
//         assert_eq!(contract.get_transaction_fee().next_fee, Some(next_fee));
//         assert_eq!(contract.get_transaction_fee().start_time, Some(start_time_sec));

//         testing_env!(context
//             .predecessor_account_id(accounts(1))
//             .block_timestamp(start_time + 1)
//             .attached_deposit(1)
//             .build()
//         );

//         contract.calculate_current_transaction_fee();
//         assert_eq!(contract.get_transaction_fee().current_fee, next_fee);
//         assert_eq!(contract.get_transaction_fee().next_fee, None);
//         assert_eq!(contract.get_transaction_fee().start_time, None);

//         let series = contract.nft_get_series_single("1".to_string());
//         let series_transaction_fee: u128 = series.transaction_fee.unwrap().into();
//         assert_eq!(series_transaction_fee, 500);
//     }
// }
