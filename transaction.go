package main

import (
  "math/big"
  "merkle-patrica-trie/common"
  "merkle-patrica-trie/rlp"
  "encoding/json"
	"io"
//   "fmt"
)

type Transaction struct {
  Type                 uint8           `json:"type,omitempty"`
  Nonce                uint64          `json:"nonce"    `
  MaxPriorityFeePerGas *big.Int        `json:"maxPriorityFeePerGas"`
  MaxFeePerGas         *big.Int        `json:"maxFeePerGas"`
  GasPrice             *big.Int        `json:"gasPrice" `
  Gas                  uint64          `json:"gas"      `
  To                   *common.Address `json:"to"       `
  Value                *big.Int        `json:"value"    `
  Data                 []byte          `json:"input"    `

  // Signature values
  V                    *big.Int        `json:"v" `
  R                    *big.Int        `json:"r" `
  S                    *big.Int        `json:"s" `

  ChainID              *big.Int        `json:"chainId,omitempty"`
  AccessList           AccessList      `json:"accessList,omitempty"`
}

// AccessList is an EIP-2930 access list.
type AccessList []AccessTuple

// AccessTuple is the element type of an access list.
type AccessTuple struct {
  Address     common.Address `json:"address"        gencodec:"required"`
  StorageKeys []common.Hash  `json:"storageKeys"    gencodec:"required"`
}

func (t Transaction) GetRLP() ([]byte, error) {
  return rlp.EncodeToBytes(t)
}

type txJSON struct {
  Type                 common.Uint64   `json:"type"`
  Nonce                *common.Uint64  `json:"nonce"`
  MaxPriorityFeePerGas *common.Big     `json:"maxPriorityFeePerGas,omitempty"`
  MaxFeePerGas         *common.Big     `json:"maxFeePerGas,omitempty"`
  GasPrice             *common.Big     `json:"gasPrice"`
  Gas                  *common.Uint64  `json:"gas"`
  To                   *common.Address `json:"to"`
  Value                *common.Big     `json:"value"`
  Data                 *common.Bytes   `json:"input"`
  V                    *common.Big     `json:"v"`
  R                    *common.Big     `json:"r"`
  S                    *common.Big     `json:"s"`

  // Access list transaction fields:
  ChainID              *common.Big    `json:"chainId,omitempty"`
  AccessList           *AccessList    `json:"accessList,omitempty"`
}

func (tx *Transaction) UnmarshalJSON(input []byte) error {
  var dec txJSON
  if err := json.Unmarshal(input, &dec); err != nil {
    return err
  }
  
  switch dec.Type {
  case LegacyTxType:
    tx.GasPrice = (*big.Int)(dec.GasPrice)
  case AccessListTxType:
    // Access list is optional for now.
    if dec.AccessList != nil {
      tx.AccessList = *dec.AccessList
    }
    tx.ChainID = (*big.Int)(dec.ChainID)
    tx.GasPrice = (*big.Int)(dec.GasPrice)
  case DynamicFeeTxType:
    // Access list is optional for now.
    if dec.AccessList != nil {
      tx.AccessList = *dec.AccessList
    }
    tx.ChainID = (*big.Int)(dec.ChainID)
    tx.MaxPriorityFeePerGas = (*big.Int)(dec.MaxPriorityFeePerGas)
    tx.MaxFeePerGas = (*big.Int)(dec.MaxFeePerGas)
  }

  tx.Type = uint8(dec.Type)
  tx.Nonce = uint64(*dec.Nonce)
  tx.Gas = uint64(*dec.Gas)
  tx.To = dec.To
  tx.Value = (*big.Int)(dec.Value)
  tx.Data = *dec.Data
  tx.V = (*big.Int)(dec.V)
  tx.R = (*big.Int)(dec.R)
  tx.S = (*big.Int)(dec.S)

  return nil
}

// Transaction types.
const (
  LegacyTxType = iota
  AccessListTxType
  DynamicFeeTxType
)

// LegacyTx is the transaction data of regular Ethereum transactions.
type LegacyTx struct {
	Nonce    uint64          // nonce of sender account
	GasPrice *big.Int        // wei per gas
	Gas      uint64          // gas limit
	To       *common.Address `rlp:"nil"` // nil means contract creation
	Value    *big.Int        // wei amount
	Data     []byte          // contract invocation input data
	V, R, S  *big.Int        // signature values
}

// AccessListTx is the data of EIP-2930 access list transactions.
type AccessListTx struct {
	ChainID    *big.Int        // destination chain ID
	Nonce      uint64          // nonce of sender account
	GasPrice   *big.Int        // wei per gas
	Gas        uint64          // gas limit
	To         *common.Address `rlp:"nil"` // nil means contract creation
	Value      *big.Int        // wei amount
	Data       []byte          // contract invocation input data
	AccessList AccessList      // EIP-2930 access list
	V, R, S    *big.Int        // signature values
}

type DynamicFeeTx struct {
	ChainID              *big.Int
	Nonce                uint64
	MaxPriorityFeePerGas *big.Int
	MaxFeePerGas         *big.Int
	Gas                  uint64
	To                   *common.Address `rlp:"nil"` // nil means contract creation
	Value                *big.Int
	Data                 []byte
	AccessList           AccessList
  V, R, S              *big.Int        // signature values
}

// EncodeRLP implements rlp.Encoder
func (tx *Transaction) EncodeRLP(w io.Writer) error {
  switch tx.Type {
  case AccessListTxType:
    data := &AccessListTx{tx.ChainID, tx.Nonce, tx.GasPrice, tx.Gas, tx.To, tx.Value, tx.Data, tx.AccessList, tx.V, tx.R, tx.S}

    return rlp.Encode(w, data)
  case DynamicFeeTxType:
    data := &DynamicFeeTx{tx.ChainID, tx.Nonce, tx.MaxPriorityFeePerGas, tx.MaxFeePerGas, tx.Gas, tx.To, tx.Value, tx.Data, tx.AccessList, tx.V, tx.R, tx.S}

    return rlp.Encode(w, data)
  default: // LegacyTxType
    data := &LegacyTx{tx.Nonce, tx.GasPrice, tx.Gas, tx.To, tx.Value, tx.Data, tx.V, tx.R, tx.S}

    return rlp.Encode(w, data)
  }
}
