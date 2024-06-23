pub mod nostr_types {
    use std::io::Read;

    use nostr_sdk::hashes::hex::DisplayHex;
    use nostr_sdk::prelude::Event as EventNostr;
    use nostr_sdk::{serde_json, JsonUtil};
    pub use parity_scale_codec::{Decode, Encode};
    use scale_info::TypeInfo;
    use serde::{Deserialize, Serialize};
    use sp_core::hexdisplay::AsBytesRef;
    use sp_runtime::offchain::storage::{MutateStorageError, StorageRetrievalError, StorageValueRef};
    use sp_runtime::offchain::storage_lock::{StorageLock, Time};
    use sp_runtime::offchain::{http, Duration};
    use sp_runtime::{BoundedVec, RuntimeDebug};
    use sp_std::vec::Vec;

    // const MAX_SIZE_VEC: usize = 32;
    pub const SIZE_VEC: usize = 32;
    // const MAX_SIZE_VEC: usize = 256;
    pub const MAX_SIZE_VEC: usize = 64;

    #[derive(
        Clone,
        Encode,
        Decode,
        Eq,
        PartialEq,
        RuntimeDebug,
        // Default,
        parity_scale_codec::MaxEncodedLen,
        TypeInfo,
    )]
    pub struct NostrEventData {
        pub id: [u8; SIZE_VEC],
        pub pubkey: [u8; SIZE_VEC],
        pub kind: u8,
        pub content: [u8; MAX_SIZE_VEC],
        // tags: [u8; SIZE_VEC],
        // tags:Vec<[u8, MAX_SIZE_VEC]>
        pub tags: [[u8; MAX_SIZE_VEC]; 0],
    }

    impl Default for NostrEventData {
        fn default() -> Self {
            Self {
                id: [0u8; SIZE_VEC],
                pubkey: [0u8; SIZE_VEC],
                kind: 0u8,
                content: [0u8; MAX_SIZE_VEC],
                // tags: [0u8; SIZE_VEC],
                // tags:vec![[0u8, MAX_SIZE_VEC.try_into().unwrap()]; 0u8]
                // tags:[[0u8, MAX_SIZE_VEC.try_into().unwrap()]; MAX_SIZE_VEC]
                // tags:[[0u8, MAX_SIZE_VEC.try_into().unwrap()]; MAX_SIZE_VEC]
                tags: [],
            }
        }
    }

    impl NostrEventData {
        pub fn from_nostr_event(event: &EventNostr) -> Result<Self, &'static str> {
            // Ensure the sizes are correct before conversion
            // if event.id.len() != SIZE_VEC {
            //     return Err("Invalid ID size");
            // }
            // if event.pubkey.len() != SIZE_VEC {
            //     return Err("Invalid public key size");
            // }
            // if event.tags.len() != SIZE_VEC {
            //     return Err("Invalid tags size");
            // }
            if event.content.len() > MAX_SIZE_VEC {
                return Err("Content size exceeds maximum allowed size");
            }

            let mut content = [0u8; MAX_SIZE_VEC];
            // content[..event.content.len()].copy_from_slice(&event.content);
            content[..event.content.len()].copy_from_slice(&event.content.as_bytes());
            // let kind = u8::try_from(event.kind.to_owned()).map_err(|_| "Kind value out of range for u8")?;
            // let kind = u8::try_from(event.kind.to_string()).map_err(|_| "Kind value out of range for u8")?;

            let kind = event.kind.as_u32() as u8;
            println!("nostr data {:?}", content);

            let nostr_data = NostrEventData {
                id: {
                    let mut array = [0u8; SIZE_VEC];
                    array.copy_from_slice(&event.id.to_bytes());
                    array
                },
                pubkey: {
                    let mut array = [0u8; SIZE_VEC];
                    array.copy_from_slice(&event.pubkey.to_bytes());
                    array
                },
                kind,
                content,
                tags: {
                    let mut array = [[0u8; MAX_SIZE_VEC]];
                    // let mut array = vec![[0u8; SIZE_VEC]];
                    // let mut array = [0u8; SIZE_VEC];

                    // array.copy_from_slice(&event.tags.to_vec());
                    // array.copy_from_slice(&event.tags.as_vec());
                    []
                },
            };
            println!("nostr data {:?}", nostr_data);

            Ok(nostr_data)
        }
    }

    const MAX_NIP05_LEN: usize = 64;
    const MAX_DISPLAY_NAME_LEN: usize = 64;
    const MAX_NAME_LEN: usize = 64;
    const MAX_WEBSITE_LEN: usize = 256;
    const MAX_BANNER_LEN: usize = 256;

    #[derive(
        // Serialize,
        // Deserialize,
        Eq,
        Encode,
        Decode,
        PartialEq,
        // RuntimeDebug,
        Clone,
        // Default,
        parity_scale_codec::MaxEncodedLen,
        TypeInfo,
        /* Default, */
    )]

    pub struct NostrUserData {
        pub id: [u8; SIZE_VEC],
        pub pubkey: [u8; SIZE_VEC],

        pub nip05: [u8; MAX_SIZE_VEC],
        pub display_name: [u8; MAX_SIZE_VEC],
        pub name: [u8; MAX_SIZE_VEC],
        pub website: [u8; MAX_SIZE_VEC],
        pub banner: [u8; MAX_SIZE_VEC],

        pub bot: bool,
        // id: Option<BoundedVec<u8, MAX_NIP05_LEN>>,
        // pubkey: Option<BoundedVec<u8, MAX_NIP05_LEN>>,
        // pub nip05: Option<BoundedVec<u8, MAX_NIP05_LEN>>,
        // pub display_name: Option<BoundedVec<u8, MAX_DISPLAY_NAME_LEN>>,
        // pub name: Option<BoundedVec<u8, MAX_NAME_LEN>>,
        // pub website: Option<BoundedVec<u8, MAX_WEBSITE_LEN>>,
        // pub banner: Option<BoundedVec<u8, MAX_BANNER_LEN>>,
        // pub bot: Option<bool>,
        // id: Option<[u8; SIZE_VEC]>,
        // // id: Option<PublicKey>,

        // pubkey: Option<[u8; SIZE_VEC]>,
        // // pubkey: Option<PublicKey>,

        // nip05: Option<Vec<u8>>,
        // display_name: Option<Vec<u8>>,
        // name: Option<Vec<u8>>,
        // website: Option<Vec<u8>>,
        // banner: Option<Vec<u8>>,
        // bot: Option<bool>,
        // nip05:Vec<u8>,
        // display_name:Vec<u8>,
        // name:Vec<u8>,
        // website:Vec<u8>,
        // banner:Vec<u8>,
        // bot:bool,
    }

    impl Default for NostrUserData {
        fn default() -> Self {
            Self {
                id: [0u8; SIZE_VEC],
                pubkey: [0u8; SIZE_VEC],
                // id: Some([0u8; SIZE_VEC]),
                // pubkey: Some([0u8; SIZE_VEC]),
                nip05: [0u8; MAX_SIZE_VEC],
                display_name: [0u8; MAX_SIZE_VEC],
                name: [0u8; MAX_SIZE_VEC],
                website: [0u8; MAX_SIZE_VEC],
                banner: [0u8; MAX_SIZE_VEC],
                // banner: Some(vec![]),
                bot: false,
            }
        }
    }

    impl NostrUserData {
        pub fn from_nostr_event_user(event: &EventNostr) -> Result<Self, &'static str> {
            // Ensure the sizes are correct before conversion
            // if event.id.len() != SIZE_VEC {
            //     return Err("Invalid ID size");
            // }
            // if event.pubkey.len() != SIZE_VEC {
            //     return Err("Invalid public key size");
            // }
            // if event.tags.len() != SIZE_VEC {
            //     return Err("Invalid tags size");
            // }
            if event.content.len() > MAX_SIZE_VEC {
                return Err("Content size exceeds maximum allowed size");
            }

            let mut content = [0u8; MAX_SIZE_VEC];
            content[..event.content.len()].copy_from_slice(&event.content.as_bytes());
            let user: NostrUser = serde_json::from_str(event.content.as_str()).unwrap();
            let nostr_user = NostrUserData {
                id: {
                    let mut array = [0u8; SIZE_VEC];
                    array.copy_from_slice(&event.id.to_bytes());
                    array
                },
                pubkey: {
                    let mut array = [0u8; SIZE_VEC];
                    array.copy_from_slice(&event.pubkey.to_bytes());
                    array
                },
                display_name: {
                    let mut array = [0u8; MAX_SIZE_VEC];
                    array.copy_from_slice(&user.display_name.unwrap().as_bytes());
                    array
                },
                banner: {
                    let mut array = [0u8; MAX_SIZE_VEC];
                    // array.copy_from_slice(&user.banner);
                    array.copy_from_slice(&user.banner.unwrap().as_bytes());

                    array
                },
                name: {
                    let mut array = [0u8; MAX_SIZE_VEC];
                    // array.copy_from_slice(&user.name);
                    array.copy_from_slice(&user.name.unwrap().as_bytes());
                    array
                },
                website: {
                    let mut array = [0u8; MAX_SIZE_VEC];
                    // array.copy_from_slice(&user.website);
                    array.copy_from_slice(&user.website.unwrap().as_bytes());
                    array
                },
                nip05: {
                    let mut array = [0u8; MAX_SIZE_VEC];
                    array.copy_from_slice(&user.nip05.unwrap().as_bytes());
                    // array.copy_from_slice(&user.nip05);
                    array
                },
                bot: { false },
             
            };
            log::info!("nostr_user nip05 {:?}", nostr_user.nip05.clone());
            log::info!("nostr_user display_name{:?}", nostr_user.display_name.clone());

            Ok(nostr_user)
        }
    }
    #[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
    pub struct NostrUser {
        pub id: Option<String>,
        pub pubkey: Option<Vec<u8>>,
        pub nip05: Option<String>,
        pub display_name: Option<String>,
        pub name: Option<String>,
        pub website: Option<String>,
        pub banner: Option<String>,
        pub bot: Option<bool>,
   
    }

    // impl Default for NostrUser {
    //     fn default() -> Self {
    //         Self {
    //             id: vec![],
    //             pubkey: vec![],
    //             nip05: vec![],
    //             display_name: vec![],
    //             name: vec![],
    //             website: vec![],
    //             banner: vec![],
    //             bot: false,
    //         }
    //     }
    // }

    impl NostrUser {
        // fn nostr_user_into_data(event: &EventNostr) -> Result<NostrUserData, &'static str>  {
        pub fn from_nostr_user_into_data(self: NostrUser) -> Result<NostrUserData, &'static str> {
            // Ensure the sizes are correct before conversion
            // if event.id.len() != SIZE_VEC {
            //     return Err("Invalid ID size");
            // }
            // if event.pubkey.len() != SIZE_VEC {
            //     return Err("Invalid public key size");
            // }
            // if event.tags.len() != SIZE_VEC {
            //     return Err("Invalid tags size");
            // }

            println!("NostrUser::from_nostr_user_into_data");
            let user = self.clone();


            let nostr_user = NostrUserData {
                id: {
                    let mut array = [0u8; SIZE_VEC];
              
                    if let Some(value) = user.id {
                        println!("The string is: {}", value);
                        array.copy_from_slice(&value.as_bytes());
                 
                    } else {
                        println!("No value found.");
                    }
                    array
                },
                pubkey: {
                    let mut array = [0u8; SIZE_VEC];
                    // array.copy_from_slice(&user.pubkey.unwrap().to_owned());
                    if let Some(value) = user.pubkey {
                        println!("The string is: {:?}", value);
                        array.copy_from_slice(&value.to_owned());

                    } else {
                        println!("No value found.");
                    }
                    // array.copy_from_slice(&user.id.unwrap().as_bytes());
                    array
                },
                display_name: {
               
                    let mut array = [0u8; MAX_SIZE_VEC];
                    if let Some(value) = user.display_name {
                        let bytes = value.as_bytes();
                        let len = bytes.len().min(MAX_SIZE_VEC); // Ensure we don't exceed the array size
                        array[..len].copy_from_slice(&bytes[..len]);
                    }

                    array
                },
                banner: {
                    let mut array = [0u8; MAX_SIZE_VEC];
                    // array.copy_from_slice(&user.banner.unwrap().as_bytes());
                    if let Some(value) = user.banner {
                        let bytes = value.as_bytes();
                        let len = bytes.len().min(MAX_SIZE_VEC); // Ensure we don't exceed the array size
                        array[..len].copy_from_slice(&bytes[..len]);
                    }
                    array
                },
                name: {
                    let mut array = [0u8; MAX_SIZE_VEC];
                    // array.copy_from_slice(&user.name.unwrap().as_bytes());
                    if let Some(value) = user.name {
                        let bytes = value.as_bytes();
                        let len = bytes.len().min(MAX_SIZE_VEC); // Ensure we don't exceed the array size
                        array[..len].copy_from_slice(&bytes[..len]);
                    }
                    array
                },
                website: {
                    let mut array = [0u8; MAX_SIZE_VEC];
                    if let Some(value) = user.website {
                        let bytes = value.as_bytes();
                        let len = bytes.len().min(MAX_SIZE_VEC); // Ensure we don't exceed the array size
                        array[..len].copy_from_slice(&bytes[..len]);
                    }
                    // array.copy_from_slice(&user.website.unwrap().as_bytes());
                    array
                },
                nip05: {
                    let mut array = [0u8; MAX_SIZE_VEC];
                    // array.copy_from_slice(&user.nip05.unwrap().as_bytes());
                    if let Some(value) = user.nip05 {
                        let bytes = value.as_bytes();
                        let len = bytes.len().min(MAX_SIZE_VEC); // Ensure we don't exceed the array size
                        array[..len].copy_from_slice(&bytes[..len]);
                    }
                    // array.copy_from_slice(&user.nip05);
                    array
                },
                bot: { false },
            };
            log::info!("nostr_user id {:?}", nostr_user.id.clone());
            log::info!("nostr_user pubkey {:?}", nostr_user.pubkey.clone());
            log::info!("nostr_user nip05 {:?}", nostr_user.nip05.clone());
            log::info!("nostr_user display_name{:?}", nostr_user.display_name.clone());

            Ok(nostr_user)
        }
    }
}
