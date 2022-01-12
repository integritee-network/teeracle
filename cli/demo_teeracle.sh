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
#  update_exchange_rate.sh -p <NODEPORT> -P <WORKERPORT> -d <DURATION> -i <WORKER_UPDATE_INTERVAL>

while getopts ":p:P:d:i:" opt; do
    case $opt in
        p)
            NPORT=$OPTARG
            ;;
        P)
            RPORT=$OPTARG
            ;;
        d)
            DURATION=$OPTARG
            ;;
        i)
            INTERVAL=$OPTARG
            ;;
    esac
done

# using default port if none given as arguments
NPORT=${NPORT:-9944}
RPORT=${RPORT:-2000}
DURATION=${DURATION:-48}
INTERVAL=${INTERVAL:-86400}

echo "Using node-port ${NPORT}"
echo "Using worker-rpc-port ${RPORT}"
echo "Using worker market data update interval ${INTERVAL}"
echo "Count the update events during ${DURATION}"
echo ""

MARKET_DATA_SRC="https://api.coingecko.com/"
let "MIN_EXPECTED_NUM_OF_EVENTS=$DURATION/$INTERVAL-1"
echo "MIN_EXPECTED_NUM_OF_EVENTS ${MIN_EXPECTED_NUM_OF_EVENTS}"

CLIENT="./../bin/integritee-cli -p ${NPORT} -P ${RPORT}"

echo "* Query on-chain enclave registry:"
${CLIENT} list-workers
echo ""

# this will always take the first MRENCLAVE found in the registry !!
read MRENCLAVE <<< $($CLIENT list-workers | awk '/  MRENCLAVE: / { print $2; exit }')
echo "Reading MRENCLAVE from worker list: ${MRENCLAVE}"

[[ -z $MRENCLAVE ]] && { echo "MRENCLAVE is empty. cannot continue" ; exit 1; }
echo ""

echo "Listen to exchange rate updated events during ${DURATION} seconds. There is no trusted data source"
${CLIENT} exchange-rate-events ${DURATION}
echo ""

read NO_EVENTS <<< $($CLIENT exchange-rate-events ${DURATION} | awk '/  EVENTS_COUNT: / { print $2; exit }')
echo "Got ${NO_EVENTS} exchange rate updates when MRENCLAVE not added in the whitelist"
echo ""

echo "Add MRENCLAVE as trusted teeracle for ${MARKET_DATA_SRC}"
${CLIENT} add-whitelist //Alice ${MARKET_DATA_SRC} ${MRENCLAVE}
echo "MRENCLAVE in Whitelist "
echo ""

echo "Listen to exchange rate updated events during ${DURATION} seconds"
${CLIENT} exchange-rate-events ${DURATION}
echo ""

read EVENTS_COUNT <<< $($CLIENT exchange-rate-events ${DURATION} | awk '/  EVENTS_COUNT: / { print $2; exit }')
echo "Got ${EVENTS_COUNT} exchange rate updates from a trusted source in ${DURATION} second"
echo ""

# the following test is for automated CI
# it only works if you're running from fresh genesis

if [ "$EVENTS_COUNT" > "$MIN_EXPECTED_NUM_OF_EVENTS" ]; then
   if [ "0" = "$NO_EVENTS" ]; then
       echo "test passed"
       exit 0
   else
       echo "test ran through but we received an exchange rate update event before the mrenclave was added to the whitelist. Have you run the script from fresh genesis?"
       exit 1
   fi
else
    echo "test failed: $MIN_EXPECTED_NUM_OF_EVENTS < ${EVENTS_COUNT} !"
    exit 1
fi

exit 0
