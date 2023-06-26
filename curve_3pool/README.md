Curve.fi: DAI/USDC/USDT Pool 
https://etherscan.io/address/0xbebc44782c7db0a1a60cb6fe97d0b483032ff1c7#code

Address
0xbEbc44782C7dB0a1A60Cb6fe97d0b483032FF1C7

//////////////////////////////////////////////////////////////////////////////

hash 0xd82caa2189d8987db426569ab12a261d93872ec472c03dac39515e3a42a4e668

TokenExchange
https://etherscan.io/tx/0xd82caa2189d8987db426569ab12a261d93872ec472c03dac39515e3a42a4e668#eventlog

Address
0xbebc44782c7db0a1a60cb6fe97d0b483032ff1c7

Name
TokenExchange (index_topic_1 address buyer, int128 sold_id, uint256 tokens_sold, int128 bought_id, uint256 tokens_bought)

Topics
0 0x8b3e96f2b889fa771c53c981b40daf005f63f637f1869f707052d15a3dd97140
1 0x000000000000000000000000d275e5cb559d6dc236a5f8002a5f0b4c8e610701 // 0xD275E5cb559D6Dc236a5f8002A5f0b4c8e610701

Data
0x000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000413b1d92dd000000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000041394366a50

sold_id: 1
tokens_sold: 4482634690000
bought_id: 2
tokens_bought: 4482137483856
//////////////////////////////////////////////////////////////////////////////
cargo run

Log {
  address: 0xbebc44782c7db0a1a60cb6fe97d0b483032ff1c7,
  topics: [0x8b3e96f2b889fa771c53c981b40daf005f63f637f1869f707052d15a3dd97140, 0x000000000000000000000000d275e5cb559d6dc236a5f8002a5f0b4c8e610701],
  data: Bytes("0x000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000413b1d92dd000000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000041394366a50"), 
  block_hash: Some(0x0e1a687089f99354adc6fbb8bece80aecbae9a4aae3c80f9f41f1cc2f4cd3445),
  block_number: Some(17535909),
  transaction_hash: Some(0xd82caa2189d8987db426569ab12a261d93872ec472c03dac39515e3a42a4e668),
  transaction_index: Some(61),
  log_index: Some(80),
  transaction_log_index: None,
  log_type: None,
  removed: Some(false)
}
//////////////////////////////////////////////////////////////////////////////

https://etherscan.io/tx/0x5b308fe171360144f6d9f72e656e387a20551e61ebde13d2511a512357c51a18#eventlog
https://etherscan.io/tx/0xec1cc57ede613965cb8322b3ab992c9feb905c31b31e80d2855605b3cc2c95a9#eventlog
...
//////////////////////////////////////////////////////////////////////////////
go test -run TestRpcTransactionsRootAndReceiptsRootAndProof -v -blockNumber=17535909

    {
        "cumulativeGasUsed": "0x347afd",
        "gasUsed": "0x21d0a",
        "logs": [
            {
                "address": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
                "data": "0x00000000000000000000000000000000000000000000000000000413b1d92dd0",
                "topics": [
                    "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef",
                    "0x000000000000000000000000d275e5cb559d6dc236a5f8002a5f0b4c8e610701",
                    "0x000000000000000000000000bebc44782c7db0a1a60cb6fe97d0b483032ff1c7"
                ]
            },
            {
                "address": "0xdac17f958d2ee523a2206206994597c13d831ec7",
                "data": "0x0000000000000000000000000000000000000000000000000000041394366a50",
                "topics": [
                    "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef",
                    "0x000000000000000000000000bebc44782c7db0a1a60cb6fe97d0b483032ff1c7",
                    "0x000000000000000000000000d275e5cb559d6dc236a5f8002a5f0b4c8e610701"
                ]
            },
            {
                "address": "0xbebc44782c7db0a1a60cb6fe97d0b483032ff1c7",
                "data": "0x000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000413b1d92dd000000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000041394366a50",
                "topics": [
                    "0x8b3e96f2b889fa771c53c981b40daf005f63f637f1869f707052d15a3dd97140",
                    "0x000000000000000000000000d275e5cb559d6dc236a5f8002a5f0b4c8e610701"
                ]
            }
        ],
        "logsBloom": "0x00000020000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000010008002008000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000410000000000000000000000000000000000000000000000000010000000000000000100000004000002000200000000080000000000000000000000000000000000000001000000002000000000000000000000000000000000000000000000000000000000000080000000000008000000000020000000000000000000000000000000080",
        "status": "0x1"
    },

go test -run TestRpcTransactionsRootAndReceiptsRootAndProof -v -blockNumber=17535192
go test -run TestRpcTransactionsRootAndReceiptsRootAndProof -v -blockNumber=17536090

//////////////////////////////////////////////////////////////////////////////
# Events
event TokenExchange:
    buyer: indexed(address)
    sold_id: int128
    tokens_sold: uint256
    bought_id: int128
    tokens_bought: uint256


@external
@nonreentrant('lock')
def exchange(i: int128, j: int128, dx: uint256, min_dy: uint256):
    assert not self.is_killed  # dev: is killed
    rates: uint256[N_COINS] = RATES

    old_balances: uint256[N_COINS] = self.balances
    xp: uint256[N_COINS] = self._xp_mem(old_balances)

    # Handling an unexpected charge of a fee on transfer (USDT, PAXG)
    dx_w_fee: uint256 = dx
    input_coin: address = self.coins[i]

    if i == FEE_INDEX:
        dx_w_fee = ERC20(input_coin).balanceOf(self)

    # "safeTransferFrom" which works for ERC20s which return bool or not
    _response: Bytes[32] = raw_call(
        input_coin,
        concat(
            method_id("transferFrom(address,address,uint256)"),
            convert(msg.sender, bytes32),
            convert(self, bytes32),
            convert(dx, bytes32),
        ),
        max_outsize=32,
    )  # dev: failed transfer
    if len(_response) > 0:
        assert convert(_response, bool)  # dev: failed transfer

    if i == FEE_INDEX:
        dx_w_fee = ERC20(input_coin).balanceOf(self) - dx_w_fee

    x: uint256 = xp[i] + dx_w_fee * rates[i] / PRECISION
    y: uint256 = self.get_y(i, j, x, xp)

    dy: uint256 = xp[j] - y - 1  # -1 just in case there were some rounding errors
    dy_fee: uint256 = dy * self.fee / FEE_DENOMINATOR

    # Convert all to real units
    dy = (dy - dy_fee) * PRECISION / rates[j]
    assert dy >= min_dy, "Exchange resulted in fewer coins than expected"

    dy_admin_fee: uint256 = dy_fee * self.admin_fee / FEE_DENOMINATOR
    dy_admin_fee = dy_admin_fee * PRECISION / rates[j]

    # Change balances exactly in same way as we change actual ERC20 coin amounts
    self.balances[i] = old_balances[i] + dx_w_fee
    # When rounding errors happen, we undercharge admin fee in favor of LP
    self.balances[j] = old_balances[j] - dy - dy_admin_fee

    # "safeTransfer" which works for ERC20s which return bool or not
    _response = raw_call(
        self.coins[j],
        concat(
            method_id("transfer(address,uint256)"),
            convert(msg.sender, bytes32),
            convert(dy, bytes32),
        ),
        max_outsize=32,
    )  # dev: failed transfer
    if len(_response) > 0:
        assert convert(_response, bool)  # dev: failed transfer

    log TokenExchange(msg.sender, i, dx, j, dy)

//////////////////////////////////////////////////////////////////////////////

// Rust web3

https://github.com/tomusdrw/rust-web3
