// #![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// pub mod weights;
// pub use weights::*;

use frame_support::pallet_prelude::DispatchResult;
use frame_system::pallet_prelude::OriginFor;

sp_api::decl_runtime_apis! {

    pub trait NostrTrait {
        // fn get_events() -> Option<T>;
        fn average_price() -> Option<u32>;
        fn submit_price(origin: OriginFor<T>, price: u32 );
        fn submit_price_unsigned(origin: OriginFor<T>,
            _block_number: BlockNumberFor<T>,
            price: u32);
        fn submit_price_unsigned_with_signed_payload(origin: OriginFor<T>,
            price_payload: PricePayload<T::Public, BlockNumberFor<T>>,
            _signature: T::Signature);
        fn save_events(origin: OriginFor<T>);

           
    }

}
