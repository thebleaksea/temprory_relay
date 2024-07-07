# temprory relay
On-demand activated cross-chain relaying based on zero-knowledge proofs

## Modified bellman
This is a modified version of the Bellman library, using the sp-std and frame-support libraries provided by Substrate to make it compatible with the no_std environment used by Substrate.

#### Key Modifications
1. Replaced some std functionalities with sp_std equivalents for compatibility with no_std.
2. Leveraged frame-support for Substrate-specific tools and macros.

## Random number generation
The original rand_core library for generating random numbers has been replaced by a custom pallet, randrng, to ensure compatibility with the no_std environment.
### Methods of realization

```rust
fn next_u32<T: Config>(&mut self) -> u32 {
    let nonce = Nonce::<T>::get();
    let (random_seed, _) = T::RandomnessSource::random(&nonce.to_le_bytes());
    let random_value = u32::from_le_bytes(random_seed.as_ref()[0..4].try_into().unwrap());
    RandomNumber::<T>::put(random_value);
    Nonce::<T>::put(nonce + 1);
    random_value
}

fn next_u64<T: Config>(&mut self) -> u64 {
    let nonce = Nonce::<T>::get();
    let (random_seed, _) = T::RandomnessSource::random(&nonce.to_le_bytes());
    let random_value = u64::from_le_bytes(random_seed.as_ref()[0..8].try_into().unwrap());
    RandomNumber64::<T>::put(random_value);
    Nonce::<T>::put(nonce + 1);
    random_value
}
```

## zero proof of knowledge
The zkproof pallet implements the three zero-knowledge proofs required for temporary relaying 'ptrade','transcation',and'store'

1. Proof of Pre-trade (`ptrade`):
This proof secures the transfer from the source chain to the temporary relay asset
2. Proof of Transaction (`transaction`):
The certificate verifies the validity of the transaction and ensures that the transaction has been confirmed by both parties
3. Proof of Storage (`store`):
This proof ensures that cross-chain results are stored correctly after the temporary relay is shut down

## Multi-Signature
To ensure secure communication, the generated zero-knowledge proofs are signed using multiple signatures before transmission. This process ensures data integrity and authenticity.

## Tips
Currently under development to further enhance the functionality and security of the temporary relay.
