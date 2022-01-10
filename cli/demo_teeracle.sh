#!/bin/bash

# setup:
# run all on localhost:
#   integritee-node purge-chain --dev
#   integritee-node --dev -lruntime=debug
#   rm light_client_db.bin
#   export RUST_LOG=integritee_service=info,ita_stf=debug
#   integritee-service init_shard
#   integritee-service shielding-key
#   integritee-service signing-key
#   integritee-service run
#
# then run this script

# usage:
#  export RUST_LOG_LOG=integritee-cli=info,ita_stf=info
#  update_exchange_rate.sh -p <NODEPORT> -P <WORKERPORT>

while getopts ":p:P:" opt; do
    case $opt in
        p)
            NPORT=$OPTARG
            ;;
        P)
            RPORT=$OPTARG
            ;;
    esac
done

# using default port if none given as arguments
NPORT=${NPORT:-9944}
RPORT=${RPORT:-2000}

echo "Using node-port ${NPORT}"
echo "Using worker-rpc-port ${RPORT}"
echo ""

MARKET_DATA_SRC="https://api.coingecko.com/"

CLIENT="./../bin/integritee-cli -p ${NPORT} -P ${RPORT}"

echo "* Query on-chain enclave registry:"
${CLIENT} list-workers
echo ""

# this will always take the first MRENCLAVE found in the registry !!
read MRENCLAVE <<< $($CLIENT list-workers | awk '/  MRENCLAVE: / { print $2; exit }')
echo "Reading MRENCLAVE from worker list: ${MRENCLAVE}"

[[ -z $MRENCLAVE ]] && { echo "MRENCLAVE is empty. cannot continue" ; exit 1; }

echo " add MRENCLAVE as trusted teeracle for src "
${CLIENT} add-whitelist //Alice ${MARKET_DATA_SRC} ${MRENCLAVE}
echo " MRENCLAVE in Whitelist "

exit 0
