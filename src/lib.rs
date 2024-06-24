mod pb;
mod utils;

use std::fs::File;
use std::io::Read;
use std::str::FromStr;

use pb::bitcoin::v1 as bitcoin;
use substreams::pb::substreams::store_delta::Operation;
use substreams::scalar::BigDecimal;
use substreams::store::{DeltaBigDecimal, StoreAdd, StoreAddBigDecimal};
use substreams::{
    log,
    store::{DeltaProto, Deltas, StoreNew, StoreSet, StoreSetProto},
    substreams_macro::{map, store}, // Import the procedural macros from the correct path
};

use substreams_entity_change::{pb::entity::EntityChanges, tables::Tables};
use substreams::errors::Error;
use utils::constants::START_BLOCK;
use utils::math::to_big_decimal;
use serde::Deserialize;

#[derive(Deserialize)]
struct BitcoinData {
    block: bitcoin::Block,
}

fn read_bitcoin_data() -> Result<bitcoin::Block, Error> {
    let mut file = File::open("src/data/bitcoin_data.json")?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;
    let bitcoin_data: BitcoinData = serde_json::from_str(&data)?;
    Ok(bitcoin_data.block)
}

#[map] // Use the correct path for the procedural macro
pub fn map_transfer() -> Result<bitcoin::Transfers, Error> {
    let block = read_bitcoin_data()?;
    Ok(bitcoin::Transfers {
        transfers: block
            .transactions
            .iter()
            .flat_map(|tx| {
                tx.outputs.iter().map(move |output| {
                    log::info!("Bitcoin transfer seen");

                    bitcoin::Transfer {
                        from: tx.inputs.iter().map(|input| input.address.clone()).collect(),
                        to: output.address.clone(),
                        block_number: block.number,
                        timestamp: block.timestamp,
                        amount: to_big_decimal(output.value.to_string().as_str())
                            .unwrap()
                            .to_string(),
                        tx_hash: tx.hash.clone(),
                        log_index: 0, // Bitcoin doesn't have log index, set to 0 or remove if not needed
                    }
                })
            })
            .collect(),
    })
}

#[store] // Use the correct path for the procedural macro
pub fn store_account_holdings(i0: bitcoin::Transfers, o: StoreAddBigDecimal) {
    for transfer in i0.transfers {
        let amount_decimal = BigDecimal::from_str(transfer.amount.as_str())
            .unwrap()
            .with_prec(10);
        for from in transfer.from {
            o.add(
                0,
                format!("Account: {}", from),
                amount_decimal.neg(),
            );
        }

        o.add(
            0,
            format!("Account: {}", transfer.to),
            amount_decimal,
        );
    }
}

#[map] // Use the correct path for the procedural macro
pub fn graph_out(
    transfers: bitcoin::Transfers,
    account_holdings: Deltas<DeltaBigDecimal>,
) -> Result<EntityChanges, Error> {
    let mut tables = Tables::new();
    for delta in account_holdings.deltas {
        let address = delta.key.as_str().split(":").last().unwrap().trim();

        match delta.operation {
            Operation::Create => {
                let row = tables.create_row("Account", address);
                row.set("holdings", delta.old_value);
            }
            Operation::Update => {
                let row = tables.update_row("Account", address);
                row.set("holdings", delta.new_value);
            }
            Operation::Delete => todo!(),
            x => panic!("unsupported operation {:?}", x),
        };
    }

    for transfer in &transfers.transfers {
        let id: String = format!("{}-{}", transfer.tx_hash, transfer.log_index);
        let row = tables.create_row("Transfer", &id);

        row.set("sender", &transfer.from.join(","));
        row.set("receiver", &transfer.to);
        row.set("timestamp", transfer.timestamp);
        row.set("blockNumber", transfer.block_number);
        row.set("txHash", &transfer.tx_hash);
        row.set("amount", &transfer.amount);
    }

    let entity_changes = tables.to_entity_changes();
    Ok(entity_changes)
}