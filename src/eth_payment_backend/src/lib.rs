mod transfer;

use b3_utils::{
    hex_string_with_0x_to_u64, log_cycle,
    memory::{
        init_stable_mem_refcell,
        types::{DefaultVMCell, DefaultVMMap},
    },
    outcall::{HttpOutcall, HttpOutcallResponse},
    u64_to_hex_string_with_0x,
};
use ic_cdk::{query, update};
use serde_json::json;
use std::cell::RefCell;
use transfer::Transfer;

thread_local! {
    static EXTERNAL_TRANSFERS: RefCell<DefaultVMMap<String, String>> = init_stable_mem_refcell("external_transfers", 1).unwrap();
    static LATEST_FETCHED_BLOCK: RefCell<DefaultVMCell<u64>> = init_stable_mem_refcell("latest_block", 2).unwrap();
}

const RECIPIENT: &str = "0xB51f94aEEebE55A3760E8169A22e536eBD3a6DCB";
const URL: &str = "https://eth-sepolia.g.alchemy.com/v2/ZpSPh3E7KZQg4mb3tN8WFXxG4Auntbxp";

#[update]
async fn get_asset_transfers(from_block: String) -> Result<HttpOutcallResponse, String> {
    let params = json!({
        "fromBlock": from_block,
        "toAddress": RECIPIENT,
        "category": ["external"],
    });

    let rpc = json!({
        "jsonrpc": "2.0",
        "id": 0,
        "method": "alchemy_getAssetTransfers",
        "params": [params]
    });

    log_cycle!("Request: {}", rpc.to_string());

    HttpOutcall::new(&URL)
        .post(&rpc.to_string(), None)
        .send_with_closure(|response: HttpOutcallResponse| HttpOutcallResponse {
            status: response.status,
            body: response.body,
            ..Default::default()
        })
        .await
}

#[query]
fn get_last_fetched_block() -> u64 {
    LATEST_FETCHED_BLOCK.with(|r| {
        let r = r.borrow();

        r.get().clone()
    })
}

#[query]
fn get_transaction_value(hash: String) -> f64 {
    EXTERNAL_TRANSFERS.with(|r| {
        let r = r.borrow();

        let transaction = r.get(&hash).unwrap().clone();

        let tx = serde_json::from_str::<Transfer>(&transaction).unwrap();

        tx.value
    })
}

#[query]
fn get_transactions() -> Vec<String> {
    EXTERNAL_TRANSFERS.with(|r| {
        let r = r.borrow();

        r.iter().map(|(_, v)| v.clone()).collect()
    })
}

#[update]
async fn get_latest_external_transfer(from_block: u64) -> u64 {
    let from_block_hex = u64_to_hex_string_with_0x(from_block);
    let result = get_asset_transfers(from_block_hex).await;

    let transfers = match result {
        Ok(response) => match serde_json::from_slice::<transfer::Root>(&response.body) {
            Ok(response_body) => {
                log_cycle!("{:?}", response_body);

                response_body.result
            }
            Err(m) => {
                panic!("Error: {}", m);
            }
        },
        Err(e) => panic!("Error: {}", e),
    };

    EXTERNAL_TRANSFERS.with(|r| {
        let mut r = r.borrow_mut();

        for transfer in transfers.transfers.iter() {
            r.insert(
                transfer.hash.clone(),
                serde_json::to_string(&transfer).unwrap(),
            );
        }
    });

    if let Some(last_transfer) = transfers.transfers.last() {
        let latest_block = hex_string_with_0x_to_u64(last_transfer.block_num.clone()).unwrap();

        LATEST_FETCHED_BLOCK.with(|r| {
            let mut r = r.borrow_mut();

            r.set(latest_block.clone()).unwrap();
        });

        latest_block
    } else {
        from_block
    }
}

ic_cdk::export_candid!();
