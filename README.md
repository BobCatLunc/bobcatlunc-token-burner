# My Swap and Burn Contract

This smart contract handles receiving LUNA, converting a portion to USTC, and then burning both assets.
Same way it can also receive USTC, swap part of it LUNC and then burn both assets.

## Usage

- **Burn LUNC**:
      Execute contract with json data
  {"receive":{}}
  and send the LUNC you wish to burn.
  The contract will swap a part of the LUNC based on the swap percent predifined into USTC
  from LUNC/USTC pool and then send both LUNC AND USTC to burn address.

- **Burn USTC**:
      Execute contract with json data
  {"receive":{}}
  and send the USTC you wish to burn.
  The contract will swap a part of the USTC based on the swap percent predifined into LUNC
  from LUNC/USTC pool and then send both LUNC AND USTC to burn address.

the swap_percent is set to 25%.
which means 75% LUNC and 25% USTC will be burned.

- **UpdateSwapPoolAddress**: Admin can update the swap pool address.
- **UpdateTaxRate**: Admin can update the tax rate to change the percent of lunc swapped to ustc.

## Deployment

run the command 
cargo wasm
to build

to run test use
cargo test


