use web3::{
    api::*,
    types::{Address,H256,Log,U256,U64},
    futures::{future, StreamExt, pin_mut},
    types::{FilterBuilder, BlockNumber, CallResult},
    transports::*,
    helpers,
    contract::{Contract, Options}
};
use std::{str::FromStr, env, collections::HashMap};
use serde::{Serialize};

#[tokio::main]
async fn main() -> web3::Result<()> {

    let transport = web3::transports::WebSocket::new(&env::var("END_POINT").unwrap()).await?; //env var defined in project/.cargo/config.toml
    let web3 = web3::Web3::new(transport);
    let aave_address:Address = Address::from_str("0x7d2768dE32b0b80b7a3454c06BdAc94A69DDc7A9").unwrap(); //Aave V2 mainnet lendingPool https://docs.aave.com/developers/v/2.0/deployed-contracts/deployed-contracts
  

    // subscribe_to_block_head(&web3).await;
    // subscribe_to_aave_liquidation(&web3, aave_address).await;
    let liq_log = scan_past_aave_liquidations(15898305,15998305, &web3, aave_address).await;
    println!("total liquidation times : {:?}", liq_log.len());
    

    // //Solution 1 - create a hashmap to store address => liquidation times, then loop it to put result back into another vector
    // let mut liq_map:HashMap<H256, u32> = HashMap::new();

    // for log in liq_log {
    //     liq_map.entry(log.topics[3]).and_modify(|liq_times| *liq_times += 1).or_insert(1);
    // }

    // // println!("{:?}", liq_map);

    // //loop the hashmap and put the final resut in a vec and sort it
    // let mut hash_vec:Vec<(H256, u32)> = Vec::new();

    // for val in liq_map {
    //     hash_vec.push(val);    
    // }

    // hash_vec.sort_by(|a,b| b.1.cmp(&a.1));

    // println!("{:?}", &hash_vec[0..5]);


    // // solution 2 - loop the vector and process the results into another vector
    
    // let liq_map:Vec<(H256, i64)> = Vec::new();

    // for log in liq_log {

    //     if(liq_map.contains((log.topics[3],_))){
    //         liq_map.push((log.topics[3], 1));
    //     }
    // }

    //Solution 3 - create a hashmap to store address => liq_record(times,value) then loop it to put result back into another vector
    let mut liq_map:HashMap<Address, (u32,u32)> = HashMap::new();
    // println!("{:?}", &liq_log[0]);
    for log in liq_log {

        //reference - aave v2 liquidation event
        //   /**
        //  * @dev Emitted when a borrower is liquidated. This event is emitted by the LendingPool via
        //  * LendingPoolCollateral manager using a DELEGATECALL
        //  * This allows to have the events in the generated ABI for LendingPool.
        //  * @param collateralAsset The address of the underlying asset used as collateral, to receive as result of the liquidation
        //  * @param debtAsset The address of the underlying borrowed asset to be repaid with the liquidation
        //  * @param user The address of the borrower getting liquidated
        //  * @param debtToCover The debt amount of borrowed `asset` the liquidator wants to cover
        //  * @param liquidatedCollateralAmount The amount of collateral received by the liiquidator
        //  * @param liquidator The address of the liquidator
        //  * @param receiveAToken `true` if the liquidators wants to receive the collateral aTokens, `false` if he wants
        //  * to receive the underlying collateral asset directly
        //  **/
        // event LiquidationCall(
        //     address indexed collateralAsset,
        //     address indexed debtAsset,
        //     address indexed user,
        //     uint256 debtToCover,
        //     uint256 liquidatedCollateralAmount,
        //     address liquidator,
        //     bool receiveAToken
        // );

        //process unindexed data out of the log.data field
        let data = helpers::serialize(&log.data);
        let data_str = data.as_str().unwrap();
        // println!("tx {:?}", data_str);
     
        //get collateral value in usd
        let collateral_asset:Address = Address::from(log.topics[1]);
        let liquidation_collateral_amount: U256 = U256::from_str_radix(&data_str[66..130], 16).unwrap();
        let collateral_value_usd = get_asset_value_usd(collateral_asset,liquidation_collateral_amount, &web3).await;
        println!("collateral value in usd {:?}", collateral_value_usd);
        
        //get debt value in usd
        let debt_to_cover: U256 = U256::from_str_radix(&data_str[2..66], 16).unwrap();
        let debt_asset:Address = Address::from(log.topics[2]);
        let debt_value_usd = get_asset_value_usd(debt_asset, debt_to_cover, &web3).await;
        println!("debt value in usd {:?}", debt_value_usd);

        //get liquidation profit
        let mut liquidation_profit:U256 = U256::from(0);

        if(collateral_value_usd > debt_value_usd){
            liquidation_profit = collateral_value_usd.checked_sub(debt_value_usd).unwrap();
        }
        println!("liquidation profit in usd {:?}", liquidation_profit); //CONTINUE HERE -> all results are not profitable. is debt_to_cover the actual amount covered?

        //get liquidator
        let liquidator:Address = Address::from_str(("0x".to_owned() + &data_str[154..194]).as_str()).unwrap();
        
        println!("==================");

        // let liquidated_collateral_amount = log.data.
        liq_map.entry(liquidator).and_modify(|liq_record| *liq_record = (liq_record.0+1, liq_record.1)).or_insert((1,0));
    }

    //loop the hashmap and put the final resut in a vec and sort it
    let mut hash_vec:Vec<(Address, (u32,u32))> = Vec::new();

    // for val:(Address, (u32, u32)) in liq_map {
    //     hash_vec.push(val);    
    // };

    hash_vec.sort_by(|a,b| b.1.cmp(&a.1));

    // println!("{:?}", &hash_vec[0..5]);

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

}

async fn get_asset_value_usd(collateral_asset:Address,liquidation_collateral_amount:U256, web3:&Web3<WebSocket>) -> U256{
    println!("getting price for {:?}", collateral_asset);
    //we just get the latest asset price from chainlink for now
    let chainlink_feed_registery = Contract::from_json(
        web3.eth(),
        Address::from_str("0x47Fb2585D2C56Fe188D0E6ec628a38b74fCeeeDf").unwrap(),
        include_bytes!("./abi/chainlinkFeedRegistryABI.json"),
    ).unwrap();

    let mut adjusted_collateral_address:Address; 

    //adjust collateral assets if it is weth and wbtc, chainlink doesn't like these two
    if (collateral_asset == Address::from_str("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2").unwrap()){
        adjusted_collateral_address = Address::from_str("0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE").unwrap();
    } else if (collateral_asset == Address::from_str("0x2260fac5e5542a773aa44fbcfedf7c193bc2c599").unwrap()){
        adjusted_collateral_address = Address::from_str("0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB").unwrap();
    } else {
        adjusted_collateral_address = collateral_asset;
    }

    let usd_address = Address::from_str("0x0000000000000000000000000000000000000348").unwrap();
    let asset_price_usd:(U256, U256, U256, U256, U256) = chainlink_feed_registery.query("latestRoundData", (adjusted_collateral_address, usd_address), None, Options::default(), None).await.unwrap();

    // println!("asset is {:?}",asset_price_usd);
    // println!("price is {:?}",asset_price_usd.1);
    // println!("price is {:?}",actual_price);

    let decimal = get_ERC20_asset_decimal(collateral_asset,&web3).await;

    let asset_decimal: U256 = U256::from(10).checked_pow(decimal).unwrap();
    // println!("asset_decimal is {:?}",asset_decimal);

    // println!("1 - {:?}",  asset_price_usd.1.checked_mul(liquidation_collateral_amount).unwrap());
    // println!("2 - {:?}",  asset_price_usd.1.checked_mul(liquidation_collateral_amount).unwrap().checked_div(U256::from_dec_str("100000000").unwrap()).unwrap());
    // println!("total value - {:?}",  asset_price_usd.1.checked_mul(liquidation_collateral_amount).unwrap().checked_div(U256::from_dec_str("100000000").unwrap()).unwrap().checked_div(asset_decimal).unwrap());

    asset_price_usd.1.checked_mul(liquidation_collateral_amount).unwrap().checked_div(U256::from_dec_str("100000000").unwrap()).unwrap().checked_div(asset_decimal).unwrap()
    
}

async fn get_ERC20_asset_decimal(asset:Address, web3:&Web3<WebSocket>) -> U256 {

    let token_contract = Contract::from_json(
        web3.eth(),
        asset,
        include_bytes!("./abi/erc20.json"),
    ).unwrap();

    let decimal:U256 = token_contract.query("decimals", (), None, Options::default(), None).await.unwrap();

    println!("decimal is {:?}", decimal);
    decimal
}