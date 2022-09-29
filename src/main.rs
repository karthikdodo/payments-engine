use std::fs::File;
use std::{collections::HashMap, env};

use crate::client::Client;
mod client;
use csv::{self, Trim};
use serde::{Deserialize, Deserializer};
use std::error::Error;

fn main() {
    let args: Vec<String> = env::args().collect();
    let file_name = &args[1];
    if let Err(e) = processpayments(file_name) {
        println!("{:?}", e);
    }
}

fn processpayments(file_name: &str) -> Result<(), Box<dyn Error>> {
    let mut map = HashMap::new();
    let mut transaction_map: HashMap<u32, f32> = HashMap::new();
    let mut dispute_list: Vec<u32> = Vec::new();
    let mut f = File::open(file_name)?;

    let mut reader = csv::ReaderBuilder::new().trim(Trim::All).from_reader(f);

    for result in reader.deserialize() {
        let record: Record = result?;
        let client_id = record.client;
        if (record.transaction_type == "deposit") {
            if !map.contains_key(&client_id) {
                add_to_map(record, &mut map, client_id, &mut transaction_map);
            } else {
                update_deposit_map(&mut map, client_id, record, &mut transaction_map);
            }
        } else if (record.transaction_type == "withdrawal") {
            if !map.contains_key(&client_id) {
                // unknown client ignore the transaction or throw an error
            } else {
                update_withdrawal_map(&mut map, client_id, record, &mut transaction_map);
            }
        } else if (record.transaction_type == "dispute") {
            update_dispute_map(
                &mut map,
                client_id,
                &transaction_map,
                record,
                &mut dispute_list,
            );
        } else if (record.transaction_type == "resolve") {
            update_resolve_map(
                &mut map,
                client_id,
                &mut dispute_list,
                record,
                &transaction_map,
            );
        } else if (record.transaction_type == "chargeback") {
            update_chargeback_map(
                &mut map,
                client_id,
                &mut dispute_list,
                record,
                &transaction_map,
            );
        }
    }

    println!("client,available,held,total,locked");
    for (key, value) in &map {
        println!(
            "{},{},{},{},{}",
            key, value.available, value.held, value.total, value.locked
        );
    }

    Ok(())
}

fn update_chargeback_map(
    map: &mut HashMap<u16, Client>,
    client_id: u16,
    dispute_list: &mut Vec<u32>,
    record: Record,
    transaction_map: &HashMap<u32, f32>,
) {
    if !map.contains_key(&client_id) {
        // unknown client ignore the transaction or throw an error
    } else if (dispute_list.contains(&record.tx)) {
        let dispute_amount = transaction_map.get(&record.tx).unwrap();
        let client = map.get(&client_id).unwrap();
        let held = client.held - dispute_amount;
        let total = client.total - dispute_amount;
        let updated_client: Client =
            get_updated_client(client_id, client.available, held, total, true);
        map.insert(client_id, updated_client);

        //remove dispute associated for this transaction - this would avoid if multiple chargeback requests are made
        let index = dispute_list.iter().position(|&x| x == record.tx).unwrap();
        dispute_list.remove(index);
    }
}

fn update_resolve_map(
    map: &mut HashMap<u16, Client>,
    client_id: u16,
    dispute_list: &mut Vec<u32>,
    record: Record,
    transaction_map: &HashMap<u32, f32>,
) {
    if !map.contains_key(&client_id) {
        // unknown client ignore the transaction or throw an error
    } else if (dispute_list.contains(&record.tx)) {
        let dispute_amount = transaction_map.get(&record.tx).unwrap();
        let client = map.get(&client_id).unwrap();
        let available = client.available + dispute_amount;
        let held: f32 = client.held - dispute_amount;
        let updated_client: Client =
            get_updated_client(client_id, available, held, client.total, client.locked);
        map.insert(client_id, updated_client);

        //remove dispute associated for this transaction - this would avoid if multiple chargeback requests are made
        let index = dispute_list.iter().position(|&x| x == record.tx).unwrap();
        dispute_list.remove(index);
    }
}

fn update_dispute_map(
    map: &mut HashMap<u16, Client>,
    client_id: u16,
    transaction_map: &HashMap<u32, f32>,
    record: Record,
    dispute_list: &mut Vec<u32>,
) {
    if !map.contains_key(&client_id) {
        // unknown client ignore the transaction or throw an error
    } else if (transaction_map.contains_key(&record.tx)) {
        let dispute_amount = transaction_map.get(&record.tx).unwrap();
        let client = map.get(&client_id).unwrap();
        let available = client.available - dispute_amount;
        let held_amount = client.held + dispute_amount;
        let updated_client: Client = get_updated_client(
            client_id,
            available,
            held_amount,
            client.total,
            client.locked,
        );
        map.insert(client_id, updated_client);
        dispute_list.push(record.tx);
    }
}

fn update_withdrawal_map(
    map: &mut HashMap<u16, Client>,
    client_id: u16,
    record: Record,
    transaction_map: &mut HashMap<u32, f32>,
) {
    let client = map.get(&client_id).unwrap();
    if (record.amount > client.available) {
        // failed transaction balance is less than requested money
    } else {
        let available = client.available - record.amount;
        let total = client.total - record.amount;
        let updated_client: Client =
            get_updated_client(client_id, available, client.held, total, client.locked);
        map.insert(client_id, updated_client);
        transaction_map.insert(record.tx, record.amount);
    }
}

fn update_deposit_map(
    map: &mut HashMap<u16, Client>,
    client_id: u16,
    record: Record,
    transaction_map: &mut HashMap<u32, f32>,
) {
    let client = map.get(&client_id).unwrap();
    let available: f32 = client.available + record.amount;
    let total: f32 = client.total + record.amount;
    let updated_client: Client =
        get_updated_client(client_id, available, client.held, total, client.locked);
    map.insert(client_id, updated_client);
    transaction_map.insert(record.tx, record.amount);
}

fn add_to_map(
    record: Record,
    map: &mut HashMap<u16, Client>,
    client_id: u16,
    transaction_map: &mut HashMap<u32, f32>,
) {
    let client: Client = Client {
        id: client_id,
        available: record.amount,
        held: 0.0000,
        total: record.amount,
        locked: false,
    };
    map.insert(client_id, client);
    transaction_map.insert(record.tx, record.amount);
}

fn get_updated_client(id: u16, available: f32, held: f32, total: f32, locked: bool) -> Client {
    let updated_client = Client {
        id: id,
        available: available,
        held: held,
        total: total,
        locked: locked,
    };
    return updated_client;
}

#[derive(Debug, Deserialize)]
struct Record {
    #[serde(rename = "type")]
    transaction_type: String,
    client: u16,
    tx: u32,
    #[serde(deserialize_with = "amount")]
    amount: f32,
}


fn amount<'de, D>(d: D) -> Result<f32, D::Error>
where
    D: Deserializer<'de>,
{
    Deserialize::deserialize(d).map(|x: Option<_>| x.unwrap_or(0.00))
}
