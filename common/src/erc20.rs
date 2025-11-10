use alloy::sol;

sol! {
    #[sol(rpc)]
    interface IERC20 {
        function symbol() external view returns (string);
        function decimals() external view returns (uint8);
        function balanceOf(address owner) external view returns (uint256);
        function approve(address spender, uint256 amount) external returns (bool);
        function transfer(address to, uint256 amount) external returns (bool);
    }
}
