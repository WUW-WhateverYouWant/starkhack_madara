// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! <!-- markdown-link-check-disable -->
//! # Offchain Worker Example Pallet
//!
//! The Offchain Worker Example: A simple pallet demonstrating
//! concepts, APIs and structures common to most offchain workers.
//!
//! Run `cargo doc --package pallet-example-offchain-worker --open` to view this module's
//! documentation.
//!
//! - [`Config`]
//! - [`Call`]
//! - [`Pallet`]
//!
//! **This pallet serves as an example showcasing Substrate off-chain worker and is not meant to
//! be used in production.**
//!
//! ## Overview
//!
//! In this example we are going to build a very simplistic, naive and definitely NOT
//! production-ready oracle for BTC/USD price.
//! Offchain Worker (OCW) will be triggered after every block, fetch the current price
//! and prepare either signed or unsigned transaction to feed the result back on chain.
//! The on-chain logic will simply aggregate the results and store last `64` values to compute
//! the average price.
//! Additional logic in OCW is put in place to prevent spamming the network with both signed
//! and unsigned transactions, and custom `UnsignedValidator` makes sure that there is only
//! one unsigned transaction floating in the network.

#![cfg_attr(not(feature = "std"), no_std)]
use std::time::Duration as StdDuration;

use frame_support::pallet_prelude::{
    InvalidTransaction as InvalidTx, TransactionValidity as TXValidity, ValidTransaction,
};
use frame_support::traits::Get;
use frame_system::offchain::{
    AppCrypto, CreateSignedTransaction, SendSignedTransaction, SendUnsignedTransaction, SignedPayload, Signer,
    SigningTypes, SubmitTransaction,
};
use frame_system::pallet_prelude::BlockNumberFor;
use frame_system::{self as system};
use lite_json::json::JsonValue;
use nostr_sdk::prelude::{Client, Event as EventNostr, EventId, Filter, JsonUtil, Keys, Kind, PublicKey};
pub use pallet::*;
// pub use pallet::types::
// use lite_json::String;
pub use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_core::bounded_vec::BoundedVec;
use sp_core::crypto::KeyTypeId;
use sp_runtime::{
    offchain::{
        http,
        storage::{MutateStorageError, StorageRetrievalError, StorageValueRef},
        storage_lock::{StorageLock, Time},
        Duration,
    },

    traits::{AtLeast32BitUnsigned, Zero},
    // BoundedVec,
    // transaction_validity::{InvalidTransaction , TransactionValidity , ValidTransaction},
    RuntimeDebug,
};
use sp_std::vec::Vec;
pub mod types;
use types::nostr_types::{NostrEventData, NostrUser, NostrUserData, SIZE_VEC};

#[cfg(test)]
mod tests;

/// Defines application identifier for crypto keys of this module.
///
/// Every module that deals with signatures needs to declare its unique identifier for
/// its crypto keys.
/// When offchain worker is signing transactions it's going to request keys of type
/// `KeyTypeId` from the keystore and use the ones it finds to sign the transaction.
/// The keys can be inserted manually via RPC (see `author_insertKey`).
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"deso");
const FETCH_INTERVAL: u64 = 10 * 60 * 1000;
/// Based on the above `KeyTypeId` we need to generate a pallet-specific crypto type wrappers.
/// We can use from supported crypto kinds (`sr25519`, `ed25519` and `ecdsa`) and augment
/// the types with this pallet-specific identifier.

pub mod crypto {
    use scale_info::prelude::string::String;
    use sp_core::sr25519::Signature as Sr25519Signature;
    use sp_runtime::app_crypto::{app_crypto, sr25519};
    use sp_runtime::traits::Verify;
    use sp_runtime::{MultiSignature, MultiSigner};

    use super::KEY_TYPE;
    app_crypto!(sr25519, KEY_TYPE);
    pub struct TestAuthId;
    impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for TestAuthId {
        type RuntimeAppPublic = Public;
        type GenericSignature = sp_core::sr25519::Signature;
        type GenericPublic = sp_core::sr25519::Public;
    }
    // implemented for mock runtime in test
    impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature> for TestAuthId {
        type RuntimeAppPublic = Public;
        type GenericSignature = sr25519::Signature;
        type GenericPublic = sr25519::Public;
    }
}

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use nostr_sdk::{serde_json, JsonUtil};
    use sp_runtime::MultiSignature;

    pub type Signature = MultiSignature;

    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// This pallet's configuration trait
    #[pallet::config]
    pub trait Config: CreateSignedTransaction<Call<Self>> + frame_system::Config {
        /// The identifier type for an offchain worker.
        type AuthorityId: AppCrypto<Self::Public, Self::Signature>;

        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        // Configuration parameters

        /// A grace period after we send transaction.
        ///
        /// To avoid sending too many transactions, we only attempt to send one
        /// every `GRACE_PERIOD` blocks. We use Local Storage to coordinate
        /// sending between distinct runs of this offchain worker.
        #[pallet::constant]
        type GracePeriod: Get<BlockNumberFor<Self>>;

        /// Number of blocks of cooldown after unsigned transaction is included.
        ///
        /// This ensures that we only accept unsigned transactions once, every `UnsignedInterval`
        /// blocks.
        #[pallet::constant]
        type UnsignedInterval: Get<BlockNumberFor<Self>>;

        /// A configuration for base priority of unsigned transactions.
        ///
        /// This is exposed so that it can be tuned for particular runtime, when
        /// multiple pallets send unsigned transactions.
        #[pallet::constant]
        type UnsignedPriority: Get<TransactionPriority>;

        /// Maximum len of event.
        #[pallet::constant]
        type MaxEventLen: Get<u32>;
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// Offchain Worker entry point.
        ///
        /// By implementing `fn offchain_worker` you declare a new offchain worker.
        /// This function will be called when the node is fully synced and a new best block is
        /// successfully imported.
        /// Note that it's not guaranteed for offchain workers to run on EVERY block, there might
        /// be cases where some blocks are skipped, or for some the worker runs twice (re-orgs),
        /// so the code should be able to handle that.
        /// You can use `Local Storage` API to coordinate runs of the worker.
        fn offchain_worker(block_number: BlockNumberFor<T>) {
            // Note that having logs compiled to WASM may cause the size of the blob to increase
            // significantly. You can use `RuntimeDebug` custom derive to hide details of the types
            // in WASM. The `sp-api` crate also provides a feature `disable-logging` to disable
            // all logging and thus, remove any logging from the WASM.
            log::info!("Offchain worker for Nostr relayer");
            // Since off-chain workers are just part of the runtime code, they have direct access
            // to the storage and other included pallets.
            //
            // We can easily import `frame_system` and retrieve a block hash of the parent block.
            let parent_hash = <system::Pallet<T>>::block_hash(block_number - 1u32.into());
            log::debug!("Current block: {:?} (parent hash: {:?})", block_number, parent_hash);
            log::info!("Nostr saved event");


            // For this example we are going to send both signed and unsigned transactions
            // depending on the block number.
            // Usually it's enough to choose one or the other.
            let should_send = Self::choose_transaction_type(block_number);

            // Check timestamp to recall offchain worker
            // let res = match should_send {
            //     TransactionType::Signed => Self::fetch_price_and_send_signed(),
            //     TransactionType::UnsignedForAny => Self::fetch_price_and_send_unsigned_for_any_account(block_number),
            //     TransactionType::UnsignedForAll => Self::fetch_price_and_send_unsigned_for_all_accounts(block_number),
            //     TransactionType::Raw => Self::fetch_price_and_send_raw_unsigned(block_number),
            //     TransactionType::None => Ok(()),
            // };
            // if let Err(e) = res {
            //     log::error!("Error: {}", e);
            // }
            let now = sp_io::offchain::timestamp().unix_millis();
            let is_need_to_fetch= Self::fetch_data_if_needed();
            log::info!("is_need_to_fetch {:?}", is_need_to_fetch);

            // Get data nostr for User and TextNote

            if !is_need_to_fetch {
                return;
            }
            let mut lock = StorageLock::<Time>::new(b"offchain-worker::lock");

            if let Ok(_guard) = lock.try_lock() {
                // Users
                let users = Self::fetch_async_events_filter(block_number, Kind::Metadata);

                //  Check if events id already exist on the storage
                match users {
                    Ok(users) => {
                        let mut events_nostr_data: Vec<NostrUser> = vec![];
                        let mut events_vec_data: Vec<Vec<u8>> = vec![];
                        for event in users.iter() {
                            let pubkey = event.pubkey.clone();
                            let content = event.content.clone();
                            log::info!("event {:?}", event);
                            log::info!("pubkey {:?}", pubkey);
                            println!("content {:?}", content);

                            let mut user: NostrUser = serde_json::from_str(content.as_str()).unwrap();
                            log::info!("user {:?}", user);

                            let mut array = [0u8; SIZE_VEC];
                            array.copy_from_slice(&event.pubkey.to_bytes());
                            user.pubkey = Some(array.to_vec());
                            if NostrUsersEvents::<T>::contains_key(pubkey.to_bytes()) {
                                log::info!("user already saved");
                            } else {
                                log::info!("user not saved at this stage");

                                let vec = event.as_json().into_bytes();
                                events_vec_data.push(vec);
                                events_nostr_data.push(user);
                            }
                        }

                        // @TODO save here the state events on a vec directly to do only one call
                        let _ = Self::store_users_nostr_signed(events_vec_data);
                    }
                    Err(_) => {}
                }

                // Event notes
                //  Check if events id already exist on the storage

                let events = Self::fetch_async_events_filter(block_number, Kind::TextNote);
                match events {
                    Ok(events) => {
                        let mut events_nostr_data: Vec<NostrEventData> = vec![];
                        let mut events_vec_data: Vec<Vec<u8>> = vec![];
                        for event in events.iter() {
                            let nostr_event_data =
                                NostrEventData::from_nostr_event(&event).map_err(|_| Error::<T>::InvalidNostrEvent); // .map_err(|_| )?;

                            match nostr_event_data {
                                Ok(nostr_data) => {
                                    // Check if the event ID already exists
                                    if NostrEvents::<T>::contains_key(&nostr_data.id) {
                                        log::info!("event already saved");
                                        // Error::<T>::EventAlreadySaved;
                                    } else {
                                        let vec = event.as_json().into_bytes();
                                        events_vec_data.push(vec);
                                        events_nostr_data.push(nostr_data);
                                    }
                                }
                                _ => {}
                            }
                        }
                        Self::save_events_signed(events_vec_data);
                    }
                    Err(_) => {}
                }

                Self::set_last_fetch_time(now);
            

            };
        }
    }

    /// A public part of the pallet.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Submit new price to the list.
        ///
        /// This method is a public function of the module and can be called from within
        /// a transaction. It appends given `price` to current list of prices.
        /// In our example the `offchain worker` will create, sign & submit a transaction that
        /// calls this function passing the price.
        ///
        /// The transaction needs to be signed (see `ensure_signed`) check, so that the caller
        /// pays a fee to execute it.
        /// This makes sure that it's not easy (or rather cheap) to attack the chain by submitting
        /// excessive transactions, but note that it doesn't ensure the price oracle is actually
        /// working and receives (and provides) meaningful data.
        /// This example is not focused on correctness of the oracle itself, but rather its
        /// purpose is to showcase offchain worker capabilities.
        #[pallet::call_index(0)]
        #[pallet::weight({0})]
        pub fn submit_price(origin: OriginFor<T>, price: u32) -> DispatchResultWithPostInfo {
            // Retrieve sender of the transaction.
            let who = ensure_signed(origin)?;
            // Add the price to the on-chain list.
            Self::add_price(Some(who), price);
            Ok(().into())
        }

        /// Submit new price to the list via unsigned transaction.
        ///
        /// Works exactly like the `submit_price` function, but since we allow sending the
        /// transaction without a signature, and hence without paying any fees,
        /// we need a way to make sure that only some transactions are accepted.
        /// This function can be called only once every `T::UnsignedInterval` blocks.
        /// Transactions that call that function are de-duplicated on the pool level
        /// via `validate_unsigned` implementation and also are rendered invalid if
        /// the function has already been called in current "session".
        ///
        /// It's important to specify `weight` for unsigned calls as well, because even though
        /// they don't charge fees, we still don't want a single block to contain unlimited
        /// number of such transactions.
        ///
        /// This example is not focused on correctness of the oracle itself, but rather its
        /// purpose is to showcase offchain worker capabilities.
        #[pallet::call_index(1)]
        #[pallet::weight({0})]
        pub fn submit_price_unsigned(
            origin: OriginFor<T>,
            _block_number: BlockNumberFor<T>,
            price: u32,
        ) -> DispatchResultWithPostInfo {
            // This ensures that the function can only be called via unsigned transaction.
            ensure_none(origin)?;
            // Add the price to the on-chain list, but mark it as coming from an empty address.
            Self::add_price(None, price);
            // now increment the block number at which we expect next unsigned transaction.
            let current_block = <system::Pallet<T>>::block_number();
            <NextUnsignedAt<T>>::put(current_block + T::UnsignedInterval::get());
            Ok(().into())
        }

        #[pallet::call_index(2)]
        #[pallet::weight({0})]
        pub fn submit_price_unsigned_with_signed_payload(
            origin: OriginFor<T>,
            price_payload: PricePayload<T::Public, BlockNumberFor<T>>,
            _signature: T::Signature,
        ) -> DispatchResultWithPostInfo {
            // This ensures that the function can only be called via unsigned transaction.
            ensure_none(origin)?;
            // Add the price to the on-chain list, but mark it as coming from an empty address.
            Self::add_price(None, price_payload.price);
            // now increment the block number at which we expect next unsigned transaction.
            let current_block = <system::Pallet<T>>::block_number();
            <NextUnsignedAt<T>>::put(current_block + T::UnsignedInterval::get());
            Ok(().into())
        }

        #[pallet::call_index(3)]
        #[pallet::weight({0})]
        pub fn store_nostr_event(
            origin: OriginFor<T>,

            event_vec: Vec<u8>, // event: EventNostr,
        ) -> DispatchResultWithPostInfo {
            log::info!("store_nostr_event");

            let who = ensure_signed(origin)?;

            // let event=event_vec.;
            let event: EventNostr = serde_json::from_slice(&event_vec).unwrap();
            // let nostr_event: Event = serde_json::from_slice(&event).map_err(|_|
            // Error::<T>::InvalidNostrEvent)?;

            // log::info!("event transform {:?}", event);

            let nostr_event_data = NostrEventData::from_nostr_event(&event);
            // log::info!("nostr_event_data {:?}", nostr_event_data);

            // Store the event in the storage map

            // Store the event in the latest storage value
            // LatestNostrEvent::<T>::put(&nostr_event_data);

            // .map_err(|_| Error::<T>::InvalidNostrEvent)?;

            match nostr_event_data {
                Ok(data) => {
                    // NostrEvents::<T>::insert(
                    //     &who, // nostr_event_data.clone()
                    //     data,
                    // );
                    NostrEvents::<T>::insert(&data.pubkey, &data);
                }
                _ => {}
            }

            // Self::deposit_event(RawEvent::NostrEventStored(who, nostr_event_data));
            // Ok(())
            Ok(().into())
        }

        #[pallet::call_index(4)]
        #[pallet::weight({0})]
        pub fn store_nostr_events(
            origin: OriginFor<T>,
            events_vec: Vec<Vec<u8>>, // event: EventNostr,
        ) -> DispatchResultWithPostInfo {
            log::info!("store_nostr_events");
            // let who = ensure_signed(origin)?;
            for event_vec in events_vec {
                let event: EventNostr = serde_json::from_slice(&event_vec).unwrap();
                // let nostr_event: Event = serde_json::from_slice(&event).map_err(|_|
                // Error::<T>::InvalidNostrEvent)?;

                // log::info!("event transform {:?}", event);

                let nostr_event_data = NostrEventData::from_nostr_event(&event);
                // .map_err(|_| Error::<T>::InvalidNostrEvent);

                // log::info!("nostr_event_data {:?}", nostr_event_data);

                // Store the event in the latest storage value

                // Store the event in the storage map

                match nostr_event_data {
                    Ok(data) => {
                        // NostrEvents::<T>::insert(
                        //     &who, // nostr_event_data.clone()
                        //     data,
                        // );
                        NostrEvents::<T>::insert(&data.id, &data);
                    }
                    _ => {}
                }
            }
            Ok(().into())
        }

        #[pallet::call_index(5)]
        #[pallet::weight({0})]
        pub fn store_users_nostr_call(
            origin: OriginFor<T>,
            events_vec: Vec<Vec<u8>>, // event: EventNostr,
        ) -> DispatchResultWithPostInfo {
            log::info!("store_users_nostr");
            // let who = ensure_signed(origin)?;
            for event_vec in events_vec {
                let event: EventNostr = serde_json::from_slice(&event_vec).unwrap();
                let pubkey = event.pubkey.clone();
                log::info!("event transform {:?}", event);
                let content = event.content.clone();
                println!("content {:?}", content);

                let json_data: &str = r#"{content}"#;

                println!("json_data {:?}", json_data);

                let user: NostrUser = serde_json::from_str(content.as_str()).unwrap();
                // let my_struct: NostrUser = serde_json::from_str(json_data).unwrap();
                // println!("user nostr display name {:#?}", user.display_name);
                // println!("user nostr nip05 name {:#?}", user.nip05);

                let mut existing_user = NostrUsersEvents::<T>::get(pubkey.to_bytes().clone());
                // .ok_or(Error::<T>::UserNotFound)?;

                // if !existing_user && Some(user).is_some() {
                //     // NostrUsersEvents::<T>::insert(pubkey.clone(), &user);
                // }

                // let new_user = user.copy();
                let new_user = user.clone();

                // let new_user_data= new_user.
                let new_user_data = NostrUser::from_nostr_user_into_data(new_user).unwrap();
                log::info!("new_user_data {:?}", new_user_data.pubkey);

                // Check if the new user data is different from the existing user data
                if existing_user == new_user_data {
                    // continue;
                    log::info!("user already saved");
                    return Err(Error::<T>::NoChangesDetected.into());
                }

                // Update the user data
                // Update fields if provided, otherwise keep existing

                existing_user.nip05 = new_user_data.nip05.clone();
                existing_user.display_name = new_user_data.display_name.clone();
                existing_user.name = new_user_data.name.clone();
                existing_user.website = new_user_data.website.clone();
                existing_user.banner = new_user_data.banner.clone();
                existing_user.bot = new_user_data.bot;
                existing_user.pubkey = new_user_data.pubkey;

                // Store the updated user in the storage map

                let nostr_event_data =
                    NostrEventData::from_nostr_event(&event).map_err(|_| Error::<T>::InvalidNostrEvent);
                // NostrUsersEvents::<T>::insert(pubkey.clone(), &existing_user);
                // NostrUsersEvents::<T>::insert(nostr_event_data.id, &existing_user);

                // Store the event in the storage map
                match nostr_event_data {
                    Ok(data) => {
                        // NostrEvents::<T>::insert(
                        //     &who, // nostr_event_data.clone()
                        //     data,
                        // );
                        println!("nostr_event_data {:?}", data.pubkey);
                        log::info!("Saved data");
                        NostrUsersEvents::<T>::insert(pubkey.to_bytes(), &existing_user);
                        NostrUsersEvents::<T>::insert(data.pubkey, &existing_user);

                        // NostrUsersEvents::<T>::insert(&data.id, &data);
                    }
                    _ => {}
                }
            }
            Ok(().into())
        }


        #[pallet::call_index(6)]
        #[pallet::weight({0})]
        pub fn save_last_fetch_time(
            origin: OriginFor<T>,
            now:u64 // event: EventNostr,
        ) -> DispatchResultWithPostInfo {
            log::info!("save_last_fetch_time");
            // let who = ensure_signed(origin)?;
         
            let times = <LastFetchTime<T>>::get();
            // <LastFetchTime<T>>::mutate(|times| {
            //     if times.try_push(now).is_err() {
            //     }
            // });
            LastFetchTime::<T>::put(now);
            // LastFetchTime::<T>::append(now);
            // LastFetchTime::<T>::push(now);
            Ok(().into())
        }

    
    }

    /// Events for the pallet.
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Event generated when new price is accepted to contribute to the average.
        NewPrice {
            price: u32,
            maybe_who: Option<T::AccountId>,
        },
        OrderCreated {
            deposit: u128,
        },
        NostrEventSaved {
            nostr_event: NostrEventData,
        },
        ListNostrEventSaved {
            nostr_event: Vec<NostrEventData>,
        },
        NostrEventStored {
            nostr_events: Vec<NostrEventData>,
        },
        // NostrEventStored(AccountId, NostrEventData),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Error names should be descriptive.
        InvalidParameter,
        /// Errors should have helpful documentation associated with them.
        OutOfSpace,
        InvalidNostrEvent,
        EventAlreadySaved,
        UserAlreadySaved,
        UserNotFound,
        NoChangesDetected,
        ExceededMaxLength,
    }

    #[pallet::validate_unsigned]
    impl<T: Config> ValidateUnsigned for Pallet<T> {
        type Call = Call<T>;

        /// Validate unsigned call to this module.
        ///
        /// By default unsigned transactions are disallowed, but implementing the validator
        /// here we make sure that some particular calls (the ones produced by offchain worker)
        /// are being whitelisted and marked as valid.
        fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TXValidity {
            // Firstly let's check that we call the right function.
            if let Call::submit_price_unsigned_with_signed_payload { price_payload: ref payload, ref signature } = call
            {
                let signature_valid = SignedPayload::<T>::verify::<T::AuthorityId>(payload, signature.clone());
                if !signature_valid {
                    return InvalidTx::BadProof.into();
                }
                Self::validate_transaction_parameters(&payload.block_number, &payload.price)
            } else if let Call::submit_price_unsigned { block_number, price: new_price } = call {
                Self::validate_transaction_parameters(block_number, new_price)
            } else {
                InvalidTx::Call.into()
            }
        }
    }

    /// A vector of recently submitted prices.
    ///
    /// This is used to calculate average price, should have bounded size.
    #[pallet::storage]
    #[pallet::getter(fn prices)]
    pub(super) type Prices<T: Config> = StorageValue<_, BoundedVec<u32, T::MaxEventLen>, ValueQuery>;

    /// This is used to calculate average price, should have bounded size.
    #[pallet::storage]
    #[pallet::getter(fn max_events_len)]
    pub(super) type MaxEventLen<T: Config> = StorageValue<_, BoundedVec<u32, T::MaxEventLen>, ValueQuery>;

    /// Defines the block when next unsigned transaction will be accepted.
    ///
    /// To prevent spam of unsigned (and unpaid!) transactions on the network,
    /// we only allow one transaction every `T::UnsignedInterval` blocks.
    /// This storage entry defines when new transaction is going to be accepted.

    #[pallet::storage]
    #[pallet::getter(fn last_fetch_time)]
    pub(super) type LastFetchTime<T: Config> = StorageValue<_, u64, ValueQuery>;
    

    #[pallet::storage]
    #[pallet::getter(fn next_unsigned_at)]
    pub(super) type NextUnsignedAt<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn nostr_events_values)]
    pub(super) type NostrEventStore<T: Config> = StorageValue<_, NostrEventData, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn users_values)]
    pub(super) type NostrUsers<T: Config> = StorageValue<_, NostrEventData, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn nostr_events)]
    pub(super) type NostrEvents<T: Config> = StorageMap<_, Blake2_128Concat, [u8; 32], NostrEventData, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn nostr_users)]
    pub(super) type NostrUsersEvents<T: Config> = StorageMap<_, Blake2_128Concat, [u8; 32], NostrUserData, ValueQuery>;
}

/// Payload used by this example crate to hold price
/// data required to submit a transaction.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
pub struct PricePayload<Public, BlockNumber> {
    block_number: BlockNumber,
    price: u32,
    public: Public,
}

impl<T: SigningTypes> SignedPayload<T> for PricePayload<T::Public, BlockNumberFor<T>> {
    fn public(&self) -> T::Public {
        self.public.clone()
    }
}

enum TransactionType {
    Signed,
    UnsignedForAny,
    UnsignedForAll,
    Raw,
    None,
}

impl<T: Config> Pallet<T> {
    /// Timestamp of last scraping
    ///   
    fn fetch_data_if_needed()-> bool {
        let now = sp_io::offchain::timestamp().unix_millis();
        let last_fetch_time = Self::last_fetch_time();
        log::info!("Now {}", now);
        log::info!("last_fetch_time {}", last_fetch_time);
        if now > last_fetch_time + FETCH_INTERVAL {
            true
            // if let Err(e) = Self::fetch_data() {
            //     log::error!("Failed to fetch data: {:?}", e);
            // } else {
            //     LastFetchTime::put(now);
            // }
        } else {
            false
        }
    }
 /// Chooses which transaction type to send.
    ///
    /// This function serves mostly to showcase `StorageValue` helper
    /// and local storage usage.
    ///
    /// Returns a type of transaction that should be produced in current run.
    ///   
    fn choose_transaction_type(block_number: BlockNumberFor<T>) -> TransactionType {
        /// A friendlier name for the error that is going to be returned in case we are in the grace
        /// period.
        const RECENTLY_SENT: () = ();

        // Start off by creating a reference to Local Storage value.
        // Since the local storage is common for all offchain workers, it's a good practice
        // to prepend your entry with the module name.
        let val = StorageValueRef::persistent(b"deso::last_send");
        // The Local Storage is persisted and shared between runs of the offchain workers,
        // and offchain workers may run concurrently. We can use the `mutate` function, to
        // write a storage entry in an atomic fashion. Under the hood it uses `compare_and_set`
        // low-level method of local storage API, which means that only one worker
        // will be able to "acquire a lock" and send a transaction if multiple workers
        // happen to be executed concurrently.
        let res = val.mutate(|last_send: Result<Option<BlockNumberFor<T>>, StorageRetrievalError>| {
            match last_send {
                // If we already have a value in storage and the block number is recent enough
                // we avoid sending another transaction at this time.
                Ok(Some(block)) if block_number < block + T::GracePeriod::get() => Err(RECENTLY_SENT),
                // In every other case we attempt to acquire the lock and send a transaction.
                _ => Ok(block_number),
            }
        });

        // The result of `mutate` call will give us a nested `Result` type.
        // The first one matches the return of the closure passed to `mutate`, i.e.
        // if we return `Err` from the closure, we get an `Err` here.
        // In case we return `Ok`, here we will have another (inner) `Result` that indicates
        // if the value has been set to the storage correctly - i.e. if it wasn't
        // written to in the meantime.
        match res {
            // The value has been set correctly, which means we can safely send a transaction now.
            Ok(block_number) => {
                // We will send different transactions based on a random number.
                // Note that this logic doesn't really guarantee that the transactions will be sent
                // in an alternating fashion (i.e. fairly distributed). Depending on the execution
                // order and lock acquisition, we may end up for instance sending two `Signed`
                // transactions in a row. If a strict order is desired, it's better to use
                // the storage entry for that. (for instance store both block number and a flag
                // indicating the type of next transaction to send).
                let transaction_type = block_number % 4u32.into();
                if transaction_type == Zero::zero() {
                    TransactionType::Signed
                } else if transaction_type == BlockNumberFor::<T>::from(1u32) {
                    TransactionType::UnsignedForAny
                } else if transaction_type == BlockNumberFor::<T>::from(2u32) {
                    TransactionType::UnsignedForAll
                } else {
                    TransactionType::Raw
                }
            }
            // We are in the grace period, we should not send a transaction this time.
            Err(MutateStorageError::ValueFunctionFailed(RECENTLY_SENT)) => TransactionType::None,
            // We wanted to send a transaction, but failed to write the block number (acquire a
            // lock). This indicates that another offchain worker that was running concurrently
            // most likely executed the same logic and succeeded at writing to storage.
            // Thus we don't really want to send the transaction, knowing that the other run
            // already did.
            Err(MutateStorageError::ConcurrentModification(_)) => TransactionType::None,
        }
    }

    fn fetch_async_events_filter(block_number: BlockNumberFor<T>, kind: Kind) -> Result<Vec<EventNostr>, &'static str> {
        log::info!("fetch async events");
        // Call the async fetch function using a synchronous block_on
        // let response = Self::block_on(Self::fetch_events(block_number));
        let response = Self::block_on(Self::fetch_events_by_kind(block_number, kind));

        match response {
            Ok(events) => Ok(events),
            Err(_) => Err("Failed to fetch data"),
        }
    }

    fn block_on<F: std::future::Future>(future: F) -> F::Output {
        use tokio::runtime::Runtime;
        let rt = Runtime::new().unwrap();
        rt.block_on(future)
    }

    /// A helper function to fetch the price, sign payload and send an unsigned transaction
    async fn fetch_events_by_kind(
        block_number: BlockNumberFor<T>,
        kind: Kind,
    ) -> Result<Vec<EventNostr>, &'static str> {
        log::info!("fetch_events");
        let my_keys = Keys::generate();
        let client = Client::new(my_keys);
        // let proxy = Some(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9050)));
        client.add_relay("wss://nostr.joyboy.community").await;

        client.connect().await;
        let filter = Filter::new().kind(kind);

        let events = client
            .get_events_of(
                vec![filter],
                Some(StdDuration::from_secs(10)), // Duration::from_millis(2_000)
            )
            .await;
        log::info!("{events:#?}");

        match events {
            Ok(data) => Ok(data),
            Err(_) => Err("Failed to fetch data"),
        }
    }

    /// A helper function to fetch the price and send signed transaction.
    fn save_event_signed(event: EventNostr) -> Result<(), &'static str> {
        let signer = Signer::<T, T::AuthorityId>::all_accounts();
        if !signer.can_sign() {
            return Err("No local accounts available. Consider adding one via `author_insertKey` RPC.");
        }

        // Using `send_signed_transaction` associated type we create and submit a transaction
        // representing the call, we've just created.
        // Submit signed will return a vector of results for all accounts that were found in the
        // local keystore with expected `KEY_TYPE`.
        let results = signer.send_signed_transaction(|_account| {
            // Received price is wrapped into a call to `submit_price` public function of this
            // pallet. This means that the transaction, when executed, will simply call that
            // function passing `price` as an argument.
            Call::store_nostr_event { event_vec: event.as_json().into_bytes() }
        });

        for (acc, res) in &results {
            match res {
                Ok(()) => {
                    // log::info!("[{:?}] Submitted events of {:?}", acc.id, event)
                }
                Err(e) => log::error!("[{:?}] Failed to submit transaction: {:?}", acc.id, e),
            }
        }

        Ok(())
    }

    /// A helper function to fetch the price and send signed transaction.
    fn save_events_signed(events: Vec<Vec<u8>>) -> Result<(), &'static str> {
        let signer = Signer::<T, T::AuthorityId>::all_accounts();
        if !signer.can_sign() {
            return Err("No local accounts available. Consider adding one via `author_insertKey` RPC.");
        }

        // Using `send_signed_transaction` associated type we create and submit a transaction
        // representing the call, we've just created.
        // Submit signed will return a vector of results for all accounts that were found in the
        // local keystore with expected `KEY_TYPE`.
        let results = signer.send_signed_transaction(|_account| {
            // Received price is wrapped into a call to `submit_price` public function of this
            // pallet. This means that the transaction, when executed, will simply call that
            // function passing `price` as an argument.
            Call::store_nostr_events { events_vec: events.clone() }
        });

        for (acc, res) in &results {
            match res {
                Ok(()) => {
                    // log::info!("[{:?}] Submitted events of {:?}", acc.id, events)
                }
                Err(e) => log::error!("[{:?}] Failed to submit transaction: {:?}", acc.id, e),
            }
        }

        Ok(())
    }
    fn set_last_fetch_time(now:u64) -> Result<(), &'static str> {
        let signer = Signer::<T, T::AuthorityId>::all_accounts();
        if !signer.can_sign() {
            return Err("No local accounts available. Consider adding one via `author_insertKey` RPC.");
        }
        // Using `send_signed_transaction` associated type we create and submit a transaction
        // representing the call, we've just created.
        // Submit signed will return a vector of results for all accounts that were found in the
        // local keystore with expected `KEY_TYPE`.
        let results = signer.send_signed_transaction(|_account| {
            // Received price is wrapped into a call to `submit_price` public function of this
            // pallet. This means that the transaction, when executed, will simply call that
            // function passing `price` as an argument.
            Call::save_last_fetch_time { now:now}
        });

        for (acc, res) in &results {
            match res {
                Ok(()) => {
                    // log::info!("[{:?}] Submitted events of {:?}", acc.id, events)
                }
                Err(e) => log::error!("[{:?}] Failed to submit transaction: {:?}", acc.id, e),
            }
        }

        Ok(())
    }


    
    /// A helper function to fetch the price and send signed transaction.
    fn store_users_nostr_signed(events: Vec<Vec<u8>>) -> Result<(), &'static str> {
        let signer = Signer::<T, T::AuthorityId>::all_accounts();
        if !signer.can_sign() {
            return Err("No local accounts available. Consider adding one via `author_insertKey` RPC.");
        }

        // Using `send_signed_transaction` associated type we create and submit a transaction
        // representing the call, we've just created.
        // Submit signed will return a vector of results for all accounts that were found in the
        // local keystore with expected `KEY_TYPE`.
        let results = signer.send_signed_transaction(|_account| {
            // Received price is wrapped into a call to `submit_price` public function of this
            // pallet. This means that the transaction, when executed, will simply call that
            // function passing `price` as an argument.
            Call::store_users_nostr_call { events_vec: events.clone() }
        });

        for (acc, res) in &results {
            match res {
                Ok(()) => {
                    // log::info!("[{:?}] Submitted users of {:?}", acc.id, events)
                }
                Err(e) => log::error!("[{:?}] Failed to submit transaction: {:?}", acc.id, e),
            }
        }

        Ok(())
    }

    /// Add new price to the list.
    fn add_price(maybe_who: Option<T::AccountId>, price: u32) {
        log::info!("Adding to the average: {}", price);
        <MaxEventLen<T>>::mutate(|prices| {
            if prices.try_push(price).is_err() {
                prices[(price % T::MaxEventLen::get()) as usize] = price;
            }
        });

        let average = Self::average_price().expect("The average is not empty, because it was just mutated; qed");
        log::info!("Current average price is: {}", average);
        // here we are raising the NewPrice event
        Self::deposit_event(Event::NewPrice { price, maybe_who });
    }

    /// Calculate current average price.
    fn average_price() -> Option<u32> {
        let prices = <Prices<T>>::get();
        if prices.is_empty() {
            None
        } else {
            Some(prices.iter().fold(0_u32, |a, b| a.saturating_add(*b)) / prices.len() as u32)
        }
    }

    fn validate_transaction_parameters(block_number: &BlockNumberFor<T>, new_price: &u32) -> TXValidity {
        // Now let's check if the transaction has any chance to succeed.
        let next_unsigned_at = <NextUnsignedAt<T>>::get();
        if &next_unsigned_at > block_number {
            return InvalidTx::Stale.into();
        }
        // Let's make sure to reject transactions from the future.
        let current_block = <system::Pallet<T>>::block_number();
        if &current_block < block_number {
            return InvalidTx::Future.into();
        }

        // We prioritize transactions that are more far away from current average.
        //
        // Note this doesn't make much sense when building an actual oracle, but this example
        // is here mostly to show off offchain workers capabilities, not about building an
        // oracle.
        let avg_price = Self::average_price()
            .map(|price| if &price > new_price { price - new_price } else { new_price - price })
            .unwrap_or(0);

        ValidTransaction::with_tag_prefix("NostrWorker")
			// We set base priority to 2**20 and hope it's included before any other
			// transactions in the pool. Next we tweak the priority depending on how much
			// it differs from the current average. (the more it differs the more priority it
			// has).
			.priority(T::UnsignedPriority::get().saturating_add(avg_price as _))
			// This transaction does not require anything else to go before into the pool.
			// In theory we could require `previous_unsigned_at` transaction to go first,
			// but it's not necessary in our case.
			//.and_requires()
			// We set the `provides` tag to be the same as `next_unsigned_at`. This makes
			// sure only one transaction produced after `next_unsigned_at` will ever
			// get to the transaction pool and will end up in the block.
			// We can still have multiple transactions compete for the same "spot",
			// and the one with higher priority will replace other one in the pool.
			.and_provides(next_unsigned_at)
			// The transaction is only valid for next 5 blocks. After that it's
			// going to be revalidated by the pool.
			.longevity(5)
			// It's fine to propagate that transaction to other peers, which means it can be
			// created even by nodes that don't produce blocks.
			// Note that sometimes it's better to keep it for yourself (if you are the block
			// producer), since for instance in some schemes others may copy your solution and
			// claim a reward.
			.propagate(true)
			.build()
    }
}
