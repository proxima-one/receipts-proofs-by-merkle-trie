package main

import (
  "math/big"

  "github.com/ethereum/go-ethereum/common"
  "github.com/ethereum/go-ethereum/common/hexutil"
  "merkle-patrica-trie/rlp"
  "encoding/json"
)

type Transaction struct {
  Nonce        uint64          `json:"nonce"    `
  GasPrice     *big.Int        `json:"gasPrice" `
  Gas          uint64          `json:"gas"      `
  To           *common.Address `json:"to"       `
  Value        *big.Int        `json:"value"    `
  Data         []byte          `json:"input"    `

  // Signature values
  V *big.Int `json:"v" `
  R *big.Int `json:"r" `
  S *big.Int `json:"s" `
}

func (t Transaction) GetRLP() ([]byte, error) {
	return rlp.EncodeToBytes(t)
}

type txJSON struct {
  Nonce    *hexutil.Uint64  `json:"nonce"`
  GasPrice *hexutil.Big     `json:"gasPrice"`
  Gas      *hexutil.Uint64  `json:"gas"`
  To       *common.Address  `json:"to"`
  Value    *hexutil.Big     `json:"value"`
  Data     *hexutil.Bytes   `json:"input"`
  V        *hexutil.Big     `json:"v"`
  R        *hexutil.Big     `json:"r"`
  S        *hexutil.Big     `json:"s"`
}

func (tx *Transaction) MarshalJSON() ([]byte, error) {
  var enc txJSON

  enc.Nonce = (*hexutil.Uint64)(&tx.Nonce)
  enc.Gas = (*hexutil.Uint64)(&tx.Gas)
  enc.GasPrice = (*hexutil.Big)(tx.GasPrice)
  enc.Value = (*hexutil.Big)(tx.Value)
  enc.Data = (*hexutil.Bytes)(&tx.Data)
  enc.To = tx.To
  enc.V = (*hexutil.Big)(tx.V)
  enc.R = (*hexutil.Big)(tx.R)
  enc.S = (*hexutil.Big)(tx.S)
  
  return json.Marshal(&enc)
}

func (tx *Transaction) UnmarshalJSON(input []byte) error {
  var dec txJSON
  if err := json.Unmarshal(input, &dec); err != nil {
    return err
  }

  tx.Nonce = uint64(*dec.Nonce)
  tx.GasPrice = (*big.Int)(dec.GasPrice)
  tx.Gas = uint64(*dec.Gas)
  tx.To = dec.To
  tx.Value = (*big.Int)(dec.Value)
  tx.Data = *dec.Data
  tx.V = (*big.Int)(dec.V)
  tx.R = (*big.Int)(dec.R)
  tx.S = (*big.Int)(dec.S)

  return nil
}
