use alloy_sol_types::sol;

sol! {
    #[sol(rpc, abi)]
    NetworkMiddlewareService,
    r#"[{"inputs":[{"internalType":"address","name":"networkRegistry","type":"address"}],"stateMutability":"nonpayable","type":"constructor"},{"inputs":[],"name":"AlreadySet","type":"error"},{"inputs":[],"name":"NotNetwork","type":"error"},{"anonymous":false,"inputs":[{"indexed":true,"internalType":"address","name":"network","type":"address"},{"indexed":false,"internalType":"address","name":"middleware","type":"address"}],"name":"SetMiddleware","type":"event"},{"inputs":[],"name":"NETWORK_REGISTRY","outputs":[{"internalType":"address","name":"","type":"address"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"address","name":"network","type":"address"}],"name":"middleware","outputs":[{"internalType":"address","name":"value","type":"address"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"address","name":"middleware_","type":"address"}],"name":"setMiddleware","outputs":[],"stateMutability":"nonpayable","type":"function"}]"#,
}
