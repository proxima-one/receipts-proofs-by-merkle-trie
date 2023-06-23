package main

import (
  "math/big"

  "merkle-patrica-trie/common"
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
  Nonce    *common.Uint64  `json:"nonce"`
  GasPrice *common.Big     `json:"gasPrice"`
  Gas      *common.Uint64  `json:"gas"`
  To       *common.Address  `json:"to"`
  Value    *common.Big     `json:"value"`
  Data     *common.Bytes   `json:"input"`
  V        *common.Big     `json:"v"`
  R        *common.Big     `json:"r"`
  S        *common.Big     `json:"s"`
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
