use alloy_sol_types::sol;

sol! {
    #[derive(Debug, PartialEq, Eq)]
    #[sol(rpc, abi)]
    interface INetworkMiddleware {
        /**
         * @notice Gets the slashing window
         */
        function SLASHING_WINDOW() external view returns (uint48);

        /**
        * @notice Gets the list of active vaults
        * @return The list of active vaults
        */
        function activeVaults() external view returns (address[] memory);

        /**
        * @notice Returns the start timestamp for a given epoch
        * @param epoch The epoch number
        * @return The start timestamp
        */
        function getEpochStart(
            uint48 epoch
        ) external view returns (uint48);

        /**
        * @notice Returns the current epoch number
        * @return The current epoch
        */
        function getCurrentEpoch() external view returns (uint48);

        /**
        * @notice Returns the duration of each epoch
        * @return The duration of each epoch
        */
        function getEpochDuration() external view returns (uint48);
    }
}
