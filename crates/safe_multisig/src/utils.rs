use crate::transaction_data::{EIP712Field, EIP712TxTypes};

// TO DO: Add version and check if its lower than 1.3.0 then use the other domain
pub fn get_eip712_tx_types() -> EIP712TxTypes {
    EIP712TxTypes {
        eip712_domain: eip712_domain(),
        safe_tx: vec![
            EIP712Field {
                field_type: "address".to_string(),
                name: "to".to_string(),
            },
            EIP712Field {
                field_type: "uint256".to_string(),
                name: "value".to_string(),
            },
            EIP712Field {
                field_type: "bytes".to_string(),
                name: "data".to_string(),
            },
            EIP712Field {
                field_type: "uint8".to_string(),
                name: "operation".to_string(),
            },
            EIP712Field {
                field_type: "uint256".to_string(),
                name: "safeTxGas".to_string(),
            },
            EIP712Field {
                field_type: "uint256".to_string(),
                name: "baseGas".to_string(),
            },
            EIP712Field {
                field_type: "uint256".to_string(),
                name: "gasPrice".to_string(),
            },
            EIP712Field {
                field_type: "address".to_string(),
                name: "gasToken".to_string(),
            },
            EIP712Field {
                field_type: "address".to_string(),
                name: "refundReceiver".to_string(),
            },
            EIP712Field {
                field_type: "uint256".to_string(),
                name: "nonce".to_string(),
            },
        ],
    }
}

fn eip712_domain_before_v130() -> Vec<EIP712Field> {
    vec![EIP712Field {
        field_type: "address".to_string(),
        name: "verifyingContract".to_string(),
    }]
}

// Constant representing the domain for v1.3.0 and later.
fn eip712_domain() -> Vec<EIP712Field> {
    vec![
        EIP712Field {
            field_type: "uint256".to_string(),
            name: "chainId".to_string(),
        },
        EIP712Field {
            field_type: "address".to_string(),
            name: "verifyingContract".to_string(),
        },
    ]
}
