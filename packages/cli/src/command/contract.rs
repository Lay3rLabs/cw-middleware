use layer_climb_cli::command::ContractLog;

pub fn handle_contract_log(log: ContractLog) {
    match log {
        ContractLog::Upload { code_id, tx_resp } => {
            println!("Contract uploaded successfully!");
            println!("Code ID: {code_id}");
            println!("Transaction Hash: {:?}", tx_resp.txhash);
        }
        ContractLog::Instantiate { addr, tx_resp } => {
            println!("Contract instantiated successfully!");
            println!("Address: {addr}");
            println!("Transaction Hash: {:?}", tx_resp.txhash);
        }
        ContractLog::Execute { tx_resp } => {
            println!("Contract executed successfully!");
            println!("Transaction Hash: {:?}", tx_resp.txhash);
        }
        ContractLog::Query { response } => {
            println!("Contract query response: {response:?}");
        }
    }
}
