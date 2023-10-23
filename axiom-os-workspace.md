## axiom-univ3-mm

RUST_BACKTRACE=1 LOOKUP_BITS=19 cargo run --example univ3_mm -- --name univ3_mm --degree 20 keygen
RUST_BACKTRACE=1 LOOKUP_BITS=19 cargo run --example univ3_mm -- --name univ3_mm --degree 20 prove
RUST_BACKTRACE=1 LOOKUP_BITS=19 cargo run --example univ3_mm -- --name univ3_mm --degree 20 verify
RUST_BACKTRACE=1 LOOKUP_BITS=19 cargo run --example univ3_mm -- --name univ3_mm --degree 20 evm
RUST_BACKTRACE=1 LOOKUP_BITS=19 cargo run --example univ3_mm -- --name univ3_mm --degree 20 mock

### Demo

`source .env && forge script script/DummyVault.s.sol:RunStrat --rpc-url $GOERLI_RPC_URL --broadcast --ffi -vvv`

`0xdaa6cd220b7d14c33500f6c60197d2212e75f5a0b525ff7bf6a8661aa4e461730xdaa6cd220b7d14c33500f6c60197d2212e75f5a0b525ff7bf6a8661aa4e46173`

Goerli Pool factory: `https://goerli.etherscan.io/address/0x1f98431c8ad98523631ae4a59f267346ea31f984#readContract`

`forge test --ffi -vvv --match-test=testDummyStrat`

`forge create --rpc-url $GOERLI_RPC_URL --private-key $PRIVATE_KEY test/mocks/MockERC20.sol:MockERC20 --etherscan-api-key $GOERLI_ETHERSCAN_KEY --verify`

Axiom USDC: https://goerli.etherscan.io/address/0x4eff99da09f7ea5acb8754c3731012ec957591fb
Axiom WETH: https://goerli.etherscan.io/address/0xf81631aedb2c5324c6dea012ac3eb181f1179e6c
Pool: https://goerli.etherscan.io/address/0x297FFb1BbAc2F906A7c8f10808E2E48825CF5b7f

`cast send --rpc-url $GOERLI_RPC_URL 0x4eff99da09f7ea5acb8754c3731012ec957591fb "mint(address,uint256)" $TEST_ADDRESS 1000000000000 --private-key=$PRIVATE_KEY`

`cast send --rpc-url $GOERLI_RPC_URL 0xf81631aedb2c5324c6dea012ac3eb181f1179e6c "mint(address,uint256)" $TEST_ADDRESS 1000000000000000000000000 --private-key=$PRIVATE_KEY`

`https://app.uniswap.org/swap?inputCurrency=0x8b5091b78200fde5ce00b57bc35ce00d788eadc5&outputCurrency=0x7a77616f628984944a578d65bb081408e132cf58`

`forge create --rpc-url $GOERLI_RPC_URL --private-key $PRIVATE_KEY contracts/DummyVault.sol:DummyVault --etherscan-api-key $GOERLI_ETHERSCAN_KEY --verify --constructor-args 0x297FFb1BbAc2F906A7c8f10808E2E48825CF5b7f 0x297FFb1BbAc2F906A7c8f10808E2E48825CF5b7f`

### TODO

- Restructure the repo.Top level dirs should be halo2-lib, contracts, axiom-sdk
