# simpleRustBot

Simple Rust bot monitering all block headers and all liquidation txs on Aave V2. 

# Setup
1. add ur own RPC endpoint: create a new file in .cargo/config.toml and add `END_POINT=wss://eth-mainnet.g.alchemy.com/v2/xxxxxxxxx` under `[env]`.
2. Cargo install

# Run program
simply run `Cargo run`   

# Expected Outputs 
live block headers and any liquidation tx in Aave V2

<img width="771" alt="Screenshot 2022-11-16 at 09 52 38" src="https://user-images.githubusercontent.com/49999458/202147960-a85d2556-63d1-4218-a294-92cbbb6b1553.png">


## Warning
Do not use for production.
