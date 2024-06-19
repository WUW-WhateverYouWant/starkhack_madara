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

use frame_support::sp_runtime::traits::Header;
use frame_support::traits::{ConstU32, ConstU64};
use frame_support::{assert_ok, parameter_types};
use frame_system::Origin;
use parity_scale_codec::Decode;
use sp_core::offchain::{testing, OffchainWorkerExt, TransactionPoolExt};
use sp_core::sr25519::Signature;
use sp_core::{H256, U256};
use sp_keystore::testing::MemoryKeystore;
use sp_keystore::{Keystore, KeystoreExt};
use sp_runtime::testing::TestXt;
use sp_runtime::traits::{BlakeTwo256, Extrinsic as ExtrinsicT, IdentifyAccount, IdentityLookup, Verify};
use sp_runtime::RuntimeAppPublic;

use crate as pallet_nostr;
use crate::*;

type Block = frame_system::mocking::MockBlock<Test>;

// For testing the module, we construct a mock runtime.
frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
        Nostr: pallet_nostr::{Pallet, Call, Storage, Event<T>, ValidateUnsigned},
    }
);

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Nonce = u64;
    // type Origin = Origin;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = sp_core::sr25519::Public;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = Block;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ConstU64<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

type Extrinsic = TestXt<RuntimeCall, ()>;
type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

impl frame_system::offchain::SigningTypes for Test {
    type Public = <Signature as Verify>::Signer;
    type Signature = Signature;
}

impl<LocalCall> frame_system::offchain::SendTransactionTypes<LocalCall> for Test
where
    RuntimeCall: From<LocalCall>,
{
    type OverarchingCall = RuntimeCall;
    type Extrinsic = Extrinsic;
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Test
where
    RuntimeCall: From<LocalCall>,
{
    fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
        call: RuntimeCall,
        _public: <Signature as Verify>::Signer,
        _account: AccountId,
        nonce: u64,
    ) -> Option<(RuntimeCall, <Extrinsic as ExtrinsicT>::SignaturePayload)> {
        Some((call, (nonce, ())))
    }
}

parameter_types! {
    pub const UnsignedPriority: u64 = 1 << 20;
}

impl Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type AuthorityId = crypto::TestAuthId;
    type GracePeriod = ConstU64<5>;
    type UnsignedInterval = ConstU64<128>;
    type UnsignedPriority = UnsignedPriority;
    type MaxEventLen = ConstU32<64>;
}

fn test_pub() -> sp_core::sr25519::Public {
    sp_core::sr25519::Public::from_raw([1u8; 32])
}

// #[test]
// fn it_aggregates_the_price() {
//     sp_io::TestExternalities::default().execute_with(|| {
//         assert_eq!(Nostr::average_price(), None);

//         assert_ok!(Nostr::submit_price(Origin::signed(Default::default()), 27));
//         assert_eq!(Nostr::average_price(), Some(27));

//         assert_ok!(Nostr::submit_price(Origin::signed(Default::default()), 43));
//         assert_eq!(Nostr::average_price(), Some(35));
//     });
// }
