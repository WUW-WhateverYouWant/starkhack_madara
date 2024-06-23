extern crate bitcoincore_rpc;

use bitcoincore_rpc::{Auth, Client, RpcApi};
use serde_json::json;
pub fn btc_client() {
    let rpc_url = "http://127.0.0.1:38332";
    let rpc_auth = Auth::UserPass("XXX".to_string(), "XXX".to_string());
    let rpc = Client::new(
       rpc_url.clone(),
        rpc_auth.clone(),
    )
    .unwrap();
    let best_block_hash = rpc.get_best_block_hash().unwrap();
    println!("best block hash: {}", best_block_hash);

    let block = rpc.get_block(&best_block_hash).unwrap();

    println!("block: {:?}", block);
    let client = Client::new(rpc_url, rpc_auth).unwrap();
    // Define the P2SH address and redeem script
    let pubkeys = [
        "03a34a1a1b6c5d1b0e67f22e3b2d35a92f003b1a14f1f2417dbacba07f41e5c696",
        "02e145f34bf1609bda8b3026f1bbf28e11f2cc87e6237084569e7db69d6bc02c52",
        "0228af9d81e7980ef1303c484f3cbe12d1048b5a0417fcd2640cc9ba01b199afc0",
    ];
    // client.create_wallet(wallet, disable_private_keys, blank, passphrase, avoid_reuse)
    let multisig = client.create_wallet(2, &pubkeys).unwrap();
    let p2sh_address = multisig.address;
    let redeem_script = multisig.redeem_script;

    println!("P2SH Address: {}", p2sh_address);
    println!("Redeem Script: {}", redeem_script);

    // Get unspent transactions
    let utxos = client.list_unspent(None, None, None, None, None).unwrap();
    let utxo = utxos.get(0).expect("No UTXOs available");

    // Define the amount and change address
    let amount_to_send = 0.1;
    let change_address = "your-change-address";

    // Create raw transaction
    let inputs = json!([{"txid": utxo.txid, "vout": utxo.vout}]);
    let outputs = json!({
        p2sh_address: amount_to_send,
        change_address: utxo.amount - amount_to_send - 0.0001 // subtract fee
    });

    let raw_tx = client
        .create_raw_transaction(&inputs, &outputs, None, None)
        .unwrap();
    println!("Raw Transaction: {}", raw_tx);

    // Sign raw transaction
    let signed_tx = client.sign_raw_transaction_with_wallet(&raw_tx).unwrap();
    let hex_signed_tx = signed_tx.hex;
    println!("Signed Transaction: {}", hex_signed_tx);

    // Send raw transaction
    let txid = client.send_raw_transaction(&hex_signed_tx).unwrap();
    println!("Transaction sent! TXID: {}", txid);

    // let wallet= rpc.create_wallet(wallet, disable_private_keys, blank, passphrase, avoid_reuse);
    // rpc.get_balance(minconf, include_watchonly);
}
