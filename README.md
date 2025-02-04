# My Swap and Burn Contract

This smart contract handles receiving LUNA, converting a portion to USTC, and then burning both assets.

## Usage

- **Receive**: Send LUNA to this contract to trigger the swap and burn action.
- **UpdateSwapPoolAddress**: Admin can update the swap pool address.
- **UpdateTaxRate**: Admin can update the tax rate to change the percent of lunc swapped to ustc.

## Deployment

run the command 
cargo wasm
to build

to run test use
cargo test