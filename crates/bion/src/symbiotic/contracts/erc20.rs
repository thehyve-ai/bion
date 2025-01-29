use alloy_sol_types::sol;

sol! {
    #[derive(Debug)]
    #[sol(rpc, abi)]
    function decimals() external view returns (uint8);

    #[derive(Debug)]
    #[sol(rpc, abi)]
    function symbol() external view returns (string);
}
