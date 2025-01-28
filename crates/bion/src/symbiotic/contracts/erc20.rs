use alloy_sol_types::sol;

sol! {
    function decimals() external view returns (uint8);
    function symbol() external view returns (string);
}
