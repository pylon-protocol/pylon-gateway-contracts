use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_slice, to_binary, Binary, Coin, ContractResult, Decimal, OwnedDeps, Querier,
    QuerierResult, QueryRequest, StdResult, SystemError, SystemResult, Uint128, WasmQuery,
};
use std::collections::HashMap;
use terra_cosmwasm::{TaxCapResponse, TaxRateResponse, TerraQuery, TerraQueryWrapper, TerraRoute};

#[allow(dead_code)]
pub fn mock_dependencies(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, CustomMockWasmQuerier> {
    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: CustomMockWasmQuerier {
            base: MockQuerier::new(&[(MOCK_CONTRACT_ADDR, contract_balance)]),
            tax: MockTax::default(),
            wasm_smart_query_handlers: HashMap::new(),
            wasm_raw_query_handlers: HashMap::new(),
        },
    }
}

pub type WasmQueryHandler = dyn Fn(&Binary) -> StdResult<Binary>;

#[derive(Clone, Default)]
pub struct MockTax {
    pub rate: Decimal,
    // this lets us iterate over all pairs that match the first string
    pub caps: HashMap<String, Uint128>,
}

impl MockTax {
    #[allow(dead_code)]
    pub fn new(rate: Decimal, caps: &[(&String, &Uint128)]) -> Self {
        let mut owner_map: HashMap<String, Uint128> = HashMap::new();
        for (denom, cap) in caps.iter() {
            owner_map.insert(denom.to_string(), **cap);
        }

        MockTax {
            rate,
            caps: owner_map,
        }
    }
}
pub struct CustomMockWasmQuerier {
    base: MockQuerier<TerraQueryWrapper>,
    tax: MockTax,
    wasm_smart_query_handlers: HashMap<String, Box<WasmQueryHandler>>,
    wasm_raw_query_handlers: HashMap<String, Box<WasmQueryHandler>>,
}

impl Querier for CustomMockWasmQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        // MockQuerier doesn't support Custom, so we ignore it completely here
        let request: QueryRequest<TerraQueryWrapper> = match from_slice(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return SystemResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {:?}", e),
                    request: bin_request.into(),
                })
            }
        };
        self.handle_query(&request)
    }
}

impl CustomMockWasmQuerier {
    #[allow(dead_code)]
    pub fn register_wasm_smart_query_handler(
        &mut self,
        address: String,
        handler: Box<WasmQueryHandler>,
    ) {
        self.wasm_smart_query_handlers.insert(address, handler);
    }

    #[allow(dead_code)]
    pub fn register_wasm_raw_query_handler(
        &mut self,
        address: String,
        handler: Box<WasmQueryHandler>,
    ) {
        self.wasm_raw_query_handlers.insert(address, handler);
    }

    fn handle_query(&self, request: &QueryRequest<TerraQueryWrapper>) -> QuerierResult {
        match request {
            QueryRequest::Custom(TerraQueryWrapper { route, query_data }) => {
                if &TerraRoute::Treasury == route {
                    match query_data {
                        TerraQuery::TaxRate {} => {
                            let res = TaxRateResponse {
                                rate: self.tax.rate,
                            };
                            SystemResult::Ok(ContractResult::Ok(to_binary(&res).unwrap()))
                        }
                        TerraQuery::TaxCap { denom } => {
                            let cap = self.tax.caps.get(denom).copied().unwrap_or_default();
                            let res = TaxCapResponse { cap };
                            SystemResult::Ok(ContractResult::Ok(to_binary(&res).unwrap()))
                        }
                        _ => panic!("DO NOT ENTER HERE"),
                    }
                } else {
                    panic!("DO NOT ENTER HERE")
                }
            }
            QueryRequest::Wasm(wasm_request) => match wasm_request {
                WasmQuery::Smart { contract_addr, msg } => SystemResult::Ok(ContractResult::Ok(
                    self.wasm_smart_query_handlers
                        .get(contract_addr.as_str())
                        .expect("wasm: smart query handler not found")(msg)
                    .unwrap(),
                )),
                WasmQuery::Raw { contract_addr, key } => SystemResult::Ok(ContractResult::Ok(
                    self.wasm_raw_query_handlers
                        .get(contract_addr.as_str())
                        .expect("wasm: raw query handler not found")(key)
                    .unwrap(),
                )),
                _ => SystemResult::Err(SystemError::UnsupportedRequest {
                    kind: stringify!(request).to_string(),
                }),
            },
            _ => self.base.handle_query(request),
        }
    }
}
