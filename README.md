> The code of this repo has been moved to the [worker repository](https://github.com/integritee-network/worker).

# integritee-teeracle

Integritee teeracle for Integritee [node](https://github.com/integritee-network/integritee-node) or [parachain](https://github.com/integritee-network/parachain).
Based on Integritee [worker](https://github.com/integritee-network/worker).

This is part of [Integritee](https://integritee.network)

The teeracle is a service that provides trustworthy exchange-rate oracles to the integritee node or parachain.

By default, it provides the TEER/USD exchange rate from CoinGecko and CoinMarketCap once a day.


## Build and Run
Similar to build and run a worker. Please see our [Integritee Book](https://book.integritee.network/howto_worker.html) to learn how to build and run a worker.

To change the update rate, run with --interval *number*s (ex --interval 28s).

To get the exchange rate from CoinMarketCap you need your own API Key. Please see [CoinMarketCap](https://coinmarketcap.com/api/documentation/v1/#). Set your API Key as environment variable COINMARKETCAP_KEY.

To start multiple worker and a node with one simple command: Check out [this README](local-setup/README.md).


## Tests
### Environment
Unit tests within the enclave can't be run by `cargo test`. All unit and integration tests can be run by the worker binary

first, you should run ipfs daemon because it is needed for testing
```
ipfs daemon
```
second, you'll need a integritee-node running
```
./target/release/integritee-node --dev --execution native
```
then you should make sure that the sealed_state is empty (but exists)
```
worker/bin$ rm sealed_stf_state.bin
worker/bin$ touch sealed_stf_state.bin
```

### Execute tests
Run these with
```
integritee-service/bin$ ./integritee-service test --all
```

### End-to-end test with benchmarking

Including cleanup between runs:

run node
```
./target/release/integritee-node purge-chain --dev
./target/release/integritee-node --dev --ws-port 9979
```

run worker

```
export RUST_LOG=debug,substrate_api_client=warn,sp_io=warn,ws=warn,integritee_service=info,enclave_runtime=info,sp_io::misc=debug,runtime=debug,enclave_runtime::state=warn,ita_stf::sgx=info,light_client=warn,rustls=warn
./integritee-service --clean-reset -r 2002 -p 9979 -w 2001 run 2>&1 | tee worker.log
```

wait until you see the worker synching a few blocks. then check MRENCLAVE and update bot-community.py constants accordingly

```
./integritee-cli -p 9979 list-workers
```

now bootstrap a new bot community

```
./bot-community.py init
./bot-community.py benchmark
```

now you should see the community growing from 10 to hundreds, increasing with every ceremony

## Direct calls scalability

For direct calls, a worker runs a web-socket server inside the enclave. An important factor for scalability is the transaction throughput of a single worker instance, which is in part defined by the maximum number of concurrent socket connections possible. On Linux by default, a process can have a maximum of `1024` concurrent file descriptors (show by `ulimit -n`).
If the web-socket server hits that limit, incoming connections will be declined until one of the established connections is closed. Permanently changing the `ulimit -n` value can be done in the `/etc/security/limits.conf` configuration file. See [this](https://linuxhint.com/permanently_set_ulimit_value/) guide for more information.
