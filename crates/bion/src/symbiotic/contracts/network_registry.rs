use alloy_sol_types::sol;

sol! {
    #[derive(Debug, PartialEq, Eq)]
    #[sol(rpc, abi)]
    interface INetworkRegistry {
        error NetworkAlreadyRegistered();
        error EntityNotExist();

        /**
         * @notice Emitted when an entity is added.
         * @param entity address of the added entity
         */
        event AddEntity(address indexed entity);

        /**
         * @notice Get if a given address is an entity.
         * @param account address to check
         * @return if the given address is an entity
         */
        function isEntity(
            address account
        ) external view returns (bool);

        /**
         * @notice Get a total number of entities.
         * @return total number of entities added
         */
        function totalEntities() external view returns (uint256);

        /**
         * @notice Get an entity given its index.
         * @param index index of the entity to get
         * @return address of the entity
         */
        function entity(
            uint256 index
        ) external view returns (address);


        /**
         * @notice Register the caller as a network.
         */
        function registerNetwork() external;
    }
}
