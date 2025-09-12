use alloy::{
    primitives::{Address, Bytes, U256},
    sol,
    sol_types::SolCall,
};

sol! {
    interface IERC20 {
        function balanceOf(address owner) external view returns (uint256);
        function transfer(address to, uint256 amount) external returns (bool);
    }
}

pub fn encode_balance_of(owner: Address) -> Bytes {
    let call = IERC20::balanceOfCall { owner };
    Bytes::from(call.abi_encode())
}

pub fn decode_balance_of(data: Bytes) -> crate::Result<U256> {
    Ok(IERC20::balanceOfCall::abi_decode_returns(&data)?)
}

pub fn encode_transfer(to: Address, amount: U256) -> Bytes {
    let transfer_call = IERC20::transferCall { to, amount };
    Bytes::from(transfer_call.abi_encode())
}
