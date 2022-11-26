use web3::{
    api::*,
    types::{Address,H256,Log},
    futures::{future, StreamExt, pin_mut},
    types::{FilterBuilder, BlockNumber, CallResult},
    transports::*,
};
use std::{str::FromStr, env, collections::HashMap};


#[tokio::main]
async fn main() -> web3::Result<()> {

    let transport = web3::transports::WebSocket::new(&env::var("END_POINT").unwrap()).await?; //env var defined in project/.cargo/config.toml
    let web3 = web3::Web3::new(transport);
    let aave_address:Address = Address::from_str("0x7d2768dE32b0b80b7a3454c06BdAc94A69DDc7A9").unwrap(); //Aave V2 mainnet lendingPool https://docs.aave.com/developers/v/2.0/deployed-contracts/deployed-contracts
  

    // subscribe_to_block_head(&web3).await;
    // subscribe_to_aave_liquidation(&web3, aave_address).await;
    let liq_log = scan_past_aave_liquidations(15898305,15998305, &web3, aave_address).await;
    println!("total liquidation times : {:?}", liq_log.len());
    
    //create a hashmap to store address => liquidation times
    let mut liq_map:HashMap<H256, u32> = HashMap::new();

    for log in liq_log {
        liq_map.entry(log.topics[3]).and_modify(|liq_times| *liq_times += 1).or_insert(1);
    }

    // println!("{:?}", liq_map);

    //loop the hashmap and put the final resut in a vec and sort it
    let mut hash_vec:Vec<(H256, u32)> = Vec::new();

    for val in liq_map {
        hash_vec.push(val);    
    }

    hash_vec.sort_by(|a,b| b.1.cmp(&a.1));

    println!("{:?}", &hash_vec[0..5]);

    Ok(())
}


async fn subscribe_to_aave_liquidation(web3: &Web3<WebSocket>,aave_address:Address) {

    println!("subscribing to aave liquidation");
   
    // Filter for liquidation event in aave v2 lendingPool contract
    let filter = FilterBuilder::default()
    .address(vec![aave_address])
    .topics(
        Some(vec![H256::from_str("e413a321e8681d831f4dbccbca790d2952b56f977908e45be37335533e005286").unwrap()]), // hash of LiquidationCall(address,address,address,uint256,uint256,address,bool)
        None,
        None,
        None,
    )
    .build();

    //listen to new blocks and print out the block info
    let sub = web3.eth_subscribe().subscribe_logs(filter).await.unwrap();

    sub.for_each(|log| {
        println!("{:?}", log);
        future::ready(())
    }).await;
}

async fn subscribe_to_block_head(web3: &Web3<WebSocket>){
    
    println!("getting to block headers");

    //subscribe to new block headers
    let sub = web3.eth_subscribe().subscribe_new_heads().await.unwrap();

    //iterate and print out block headers
    sub.for_each(|header|{
        println!("got new block hash {:?}", header.unwrap().hash.unwrap());
        future::ready(())
    }).await;

}

async fn scan_past_aave_liquidations(from_block_num:u64, to_block_num:u64, web3: &Web3<WebSocket>, aave_address:Address) -> Vec<Log>{
    println!("start");

    let filter = FilterBuilder::default().from_block(BlockNumber::from(from_block_num)).to_block(BlockNumber::from(to_block_num))
    .address(vec![aave_address])
    .topics(
        Some(vec![H256::from_str("e413a321e8681d831f4dbccbca790d2952b56f977908e45be37335533e005286").unwrap()]), // hash of LiquidationCall(address,address,address,uint256,uint256,address,bool)
        None,
        None,
        None,
    )
    .build();

    let filter = web3.eth_filter().create_logs_filter(filter).await.unwrap();

    let logs_stream = filter.logs().await.unwrap();
    return logs_stream;

    // for log in logs_stream.iter() {
    //     println!("{:?}",log);
    // }


}