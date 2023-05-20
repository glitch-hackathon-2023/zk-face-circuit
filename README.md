# BYOF Halo2 Circuit (ง'̀-'́)ง
## We're freakin' kicking off our first rendezvous on Halo2 together 🏃🏻🏃🏼🏃🏽🏃🏾🏃

### Environment
* Python 3.10 (poetry, maturin) & rustup 1.26.0 (5af9b9484 2023-04-05)
```shell
poetry install
poetry shell
cd voice_recovery_python
maturin develop
```
* Export the Halo2 circuit to Solidity Verifier
  * You need to run the command below from the root directory of this repository.
  * Please make sure that you have
```
cargo run gen-params --k 20
cargo run gen-keys
cargo run gen-evm-verifier
```