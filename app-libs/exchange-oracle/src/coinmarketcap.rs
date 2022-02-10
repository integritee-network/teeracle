/*
	Copyright 2021 Integritee AG and Supercomputing Systems AG

	Licensed under the Apache License, Version 2.0 (the "License");
	you may not use this file except in compliance with the License.
	You may obtain a copy of the License at

		http://www.apache.org/licenses/LICENSE-2.0

	Unless required by applicable law or agreed to in writing, software
	distributed under the License is distributed on an "AS IS" BASIS,
	WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
	See the License for the specific language governing permissions and
	limitations under the License.

*/
#[cfg(all(not(feature = "std"), feature = "sgx"))]
use crate::sgx_reexport_prelude::*;

use crate::{
	error::Error,
	types::{TradingPair, TradingPairId},
	ExchangeRate, GetExchangeRate,
};
use itc_rest_client::{http_client::HttpClient, rest_client::RestClient, RestGet, RestPath};
use lazy_static::lazy_static;
use log::*;
use serde::{Deserialize, Serialize};
use std::{
	collections::HashMap,
	string::{String, ToString},
	time::Duration,
};
use url::Url;

const COINMARKETCAP_URL: &str = "https://pro-api.coinmarketcap.com";
const COINMARKETCAP_KEY: &str = "67db2bd8-ac38-4609-84fe-de9af99d9720"; //Has to be changed regenerate.
const COINMARKETCAP_KEY_PARAM: &str = "CMC_PRO_API_KEY"; //Has to be changed regenerate.
const FIAT_CURRENCY_PARAM: &str = "convert_id";
const CRYPTO_CURRENCY_PARAM: &str = "id";
const COINMARKETCAP_PATH: &str = "v2/cryptocurrency/quotes/latest";
//const COINMARKETCAP_PATH: &str = "v1/tools/price-conversion";
const COINMARKETCAP_TIMEOUT: Duration = Duration::from_secs(3u64);

//From https://pro-api.coinmarketcap.com/v1/cryptocurrency/map?CMC_PRO_API_KEY=COINMARKETCAP_KEY
lazy_static! {
	static ref CRYPTO_SYMBOL_ID_MAP: HashMap<&'static str, &'static str> = HashMap::from([
		("DOT", "6636"),//id":6636,"name":"Polkadot","symbol":"DOT"
		("TEER", "13323"), //"id":13323,"name":"Integritee Network","symbol":"TEER"
		("KSM", "5034"), //"id":5034,"name":"Kusama","symbol":"KSM"
		("BTC", "1"), //"id":1,"name":"Bitcoin","symbol":"BTC"
	]);
}

//From https://pro-api.coinmarketcap.com/v1/fiat/map?CMC_PRO_API_KEY=COINMARKETCAP_KEY
lazy_static! {
	static ref FIAT_SYMBOL_ID_MAP: HashMap<&'static str, &'static str> =
		HashMap::from([("USD", "2781"), ("EUR", "2790"), ("CHF", "2785"), ("JPY", "2797"),]);
}
//FOR free plan:
//https://pro-api.coinmarketcap.com/v1/cryptocurrency/quotes/latest?id=13323&convert_id=2781&CMC_PRO_API_KEY=67db2bd8-ac38-4609-84fe-de9af99d9720
//id=13323, 5034, .... but only 1 convert options
//{"status":{"timestamp":"2022-01-13T17:43:34.270Z","error_code":0,"error_message":null,"elapsed":30,"credit_count":1,"notice":null},"data":{"13323":{"id":13323,"name":"Integritee Network","symbol":"TEER","slug":"integritee-network","num_market_pairs":1,"date_added":"2021-10-26T15:04:11.000Z","tags":[],"max_supply":10000000,"circulating_supply":0,"total_supply":10000000,"is_active":1,"platform":null,"cmc_rank":4900,"is_fiat":0,"last_updated":"2022-01-13T17:42:00.000Z","quote":{"2781":{"price":4.967298630193371,"volume_24h":72743.61342986,"volume_change_24h":182.2754,"percent_change_1h":-1.00275087,"percent_change_24h":24.14973926,"percent_change_7d":54.10997674,"percent_change_30d":47.44551742,"percent_change_60d":2.02125024,"percent_change_90d":2.02125024,"market_cap":0,"market_cap_dominance":0,"fully_diluted_market_cap":49672986.3,"last_updated":"2022-01-13T17:42:00.000Z"}}}}}

/// REST client to make requests to CoinMarketCap.
pub struct CoinMarketCapClient {
	client: RestClient<HttpClient>,
}
impl CoinMarketCapClient {
	pub fn new(baseurl: Url) -> Self {
		let http_client = HttpClient::new(true, Some(COINMARKETCAP_TIMEOUT), None, None);
		let rest_client = RestClient::new(http_client, baseurl);
		CoinMarketCapClient { client: rest_client }
	}
	pub fn base_url() -> Result<Url, Error> {
		Url::parse(COINMARKETCAP_URL).map_err(|e| Error::Other(format!("{:?}", e).into()))
	}
}

impl TradingPairId for CoinMarketCapClient {
	fn crypto_currency_id(&mut self, trading_pair: TradingPair) -> Result<String, Error> {
		let key = trading_pair.crypto_currency;
		match CRYPTO_SYMBOL_ID_MAP.get(&key as &str) {
			Some(v) => Ok(v.to_string()),
			None => Err(Error::InvalidCryptoCurrencyId),
		}
	}

	fn fiat_currency_id(&mut self, trading_pair: TradingPair) -> Result<String, Error> {
		let key = trading_pair.fiat_currency;
		match FIAT_SYMBOL_ID_MAP.get(&key as &str) {
			Some(v) => Ok(v.to_string()),
			None => Err(Error::InvalidFiatCurrencyId),
		}
	}
}
#[derive(Serialize, Deserialize, Debug)]
pub struct DataStruct {
	id: Option<u32>,
	name: String,
	symbol: String,
	quote: HashMap<String, QuoteStruct>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QuoteStruct {
	price: Option<f32>,
	last_updated: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CoinMarketCapMarketStruct {
	data: HashMap<String, DataStruct>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct CoinMarketCapMarket(pub CoinMarketCapMarketStruct);

impl RestPath<String> for CoinMarketCapMarket {
	fn get_path(path: String) -> Result<String, itc_rest_client::error::Error> {
		Ok(path)
	}
}

impl GetExchangeRate for CoinMarketCapClient {
	fn get_exchange_rate(&mut self, trading_pair: TradingPair) -> Result<ExchangeRate, Error> {
		let fiat_id = self.fiat_currency_id(trading_pair.clone())?;
		let crypto_id = self.crypto_currency_id(trading_pair.clone())?;
		let response = self
			.client
			.get_with::<String, CoinMarketCapMarket>(
				COINMARKETCAP_PATH.to_string(),
				&[
					(FIAT_CURRENCY_PARAM, &fiat_id),
					(CRYPTO_CURRENCY_PARAM, &crypto_id),
					(COINMARKETCAP_KEY_PARAM, COINMARKETCAP_KEY),
				],
			)
			.map_err(Error::RestClient)?;

		let data_struct = response.0;

		println!("Got data struct {:?}", data_struct);

		let data = match data_struct.data.get(&crypto_id) {
			Some(d) => d,
			None => {
				error!("Got no market data from CoinMarketCap for {} ", crypto_id);
				return Err(Error::NoValidData)
			},
		};

		let quote = match data.quote.get(&fiat_id) {
			Some(q) => q,
			None => {
				error!("Got no market data from CoinMarketCap. Check params {:?} ", trading_pair);
				return Err(Error::NoValidData)
			},
		};
		match quote.price {
			Some(r) => Ok(ExchangeRate::from_num(r)),
			None => {
				error!("Failed to get the exchange rate {}", TradingPair::key(trading_pair));
				Err(Error::EmptyExchangeRate)
			},
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use core::assert_matches::assert_matches;
	use substrate_fixed::transcendental::ZERO;

	fn get_coinmarketcap_crypto_currency_id(crypto_currency: &str) -> Result<String, Error> {
		let url = CoinMarketCapClient::base_url().unwrap();
		let mut coinmarketcap_client = CoinMarketCapClient::new(url);
		let trading_pair = TradingPair {
			crypto_currency: crypto_currency.to_string(),
			fiat_currency: "USD".to_string(),
		};
		coinmarketcap_client.crypto_currency_id(trading_pair)
	}

	#[test]
	fn crypto_currency_id_works_for_dot() {
		let coin_id = get_coinmarketcap_crypto_currency_id("DOT").unwrap();
		assert_eq!(&coin_id, "6636");
	}

	#[test]
	fn crypto_currency_id_works_for_teer() {
		let coin_id = get_coinmarketcap_crypto_currency_id("TEER").unwrap();
		assert_eq!(&coin_id, "13323");
	}

	#[test]
	fn crypto_currency_id_works_for_ksm() {
		let coin_id = get_coinmarketcap_crypto_currency_id("KSM").unwrap();
		assert_eq!(&coin_id, "5034");
	}

	#[test]
	fn crypto_currency_id_works_for_btc() {
		let coin_id = get_coinmarketcap_crypto_currency_id("BTC").unwrap();
		assert_eq!(&coin_id, "1");
	}

	#[test]
	fn crypto_currency_id_fails_for_undefined_crypto_currency() {
		let coin_id = get_coinmarketcap_crypto_currency_id("Undefined");
		assert_matches!(coin_id, Err(Error::InvalidCryptoCurrencyId));
	}

	#[test]
	fn get_exchange_rate_for_undefined_coinmarketcap_crypto_currency_fails() {
		let url = CoinMarketCapClient::base_url().unwrap();
		let mut coinmarketcap_client = CoinMarketCapClient::new(url);
		let trading_pair = TradingPair {
			crypto_currency: "invalid_coin".to_string(),
			fiat_currency: "USD".to_string(),
		};
		let result = coinmarketcap_client.get_exchange_rate(trading_pair);
		assert_matches!(result, Err(Error::InvalidCryptoCurrencyId));
	}

	#[test]
	fn get_exchange_rate_for_undefined_fiat_currency_fails() {
		let url = CoinMarketCapClient::base_url().unwrap();
		let mut coinmarketcap_client = CoinMarketCapClient::new(url);
		let trading_pair =
			TradingPair { crypto_currency: "DOT".to_string(), fiat_currency: "CH".to_string() };
		let result = coinmarketcap_client.get_exchange_rate(trading_pair);
		assert_matches!(result, Err(Error::InvalidFiatCurrencyId));
	}

	#[test]
	fn get_exchange_rate_from_coinmarketcap_works() {
		let url = CoinMarketCapClient::base_url().unwrap();
		let mut coinmarketcap_client = CoinMarketCapClient::new(url);
		let dot_to_usd =
			TradingPair { crypto_currency: "DOT".to_string(), fiat_currency: "USD".to_string() };
		let dot_usd = coinmarketcap_client.get_exchange_rate(dot_to_usd).unwrap();
		assert!(dot_usd > 0f32);
		let btc_to_usd =
			TradingPair { crypto_currency: "BTC".to_string(), fiat_currency: "USD".to_string() };
		let bit_usd = coinmarketcap_client.get_exchange_rate(btc_to_usd).unwrap();
		assert!(bit_usd > 0f32);
		let dot_to_chf =
			TradingPair { crypto_currency: "DOT".to_string(), fiat_currency: "CHF".to_string() };
		let dot_chf = coinmarketcap_client.get_exchange_rate(dot_to_chf).unwrap();
		assert!(dot_chf > 0f32);
		let bit_to_chf =
			TradingPair { crypto_currency: "BTC".to_string(), fiat_currency: "CHF".to_string() };
		let bit_chf = coinmarketcap_client.get_exchange_rate(bit_to_chf).unwrap();

		//Ensure that get_exchange_rate return a positive rate
		assert!(dot_usd > ZERO);
		//Ensure that the exchange rates' values make sense
		assert_eq!((dot_usd / bit_usd).round(), (dot_chf / bit_chf).round());
	}
}
