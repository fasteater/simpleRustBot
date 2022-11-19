use web3::{
    api::*,
    types::{Address,H256},
    futures::{future, StreamExt},
    types::FilterBuilder,
    transports::*,
};
use std::{str::FromStr, env};


#[tokio::main]
async fn main() -> web3::Result<()> {
    println!("{:?}", env::var("END_POINT"));
    let transport = web3::transports::WebSocket::new(&env::var("END_POINT").unwrap()).await?; //env var defined in project/.cargo/config.toml
    let web3 = web3::Web3::new(transport);

    subscribe_to_block_head(&web3).await;
    subscribe_to_aave_liquidation(&web3).await;

    Ok(())
}


async fn subscribe_to_aave_liquidation(web3: &Web3<WebSocket>) {

    println!("subscribing to aave liquidation");

    let aave_address:Address = Address::from_str("0xbd4765210d4167CE2A5b87280D9E8Ee316D5EC7C").unwrap(); //Aave V2 mainnet lendingPoolCollateralManager https://docs.aave.com/developers/v/2.0/deployed-contracts/deployed-contracts
    
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