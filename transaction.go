package main

import (
  "math/big"
	"errors"
  "merkle-patrica-trie/common"
  "merkle-patrica-trie/rlp"
  "encoding/json"
	"bytes"
	"io"
//   "fmt"
)

var (
	ErrInvalidSig           = errors.New("invalid transaction v, r, s values")
	ErrUnexpectedProtection = errors.New("transaction type does not supported EIP-155 protected signatures")
	ErrInvalidTxType        = errors.New("transaction type not valid in this context")
	ErrTxTypeNotSupported   = errors.New("transaction type not supported")
	ErrGasFeeCapTooLow      = errors.New("fee cap less than base fee")
	errShortTypedTx         = errors.New("typed transaction too short")
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

// StorageKeys returns the total number of storage keys in the access list.
func (al AccessList) StorageKeys() int {
  sum := 0
  for _, tuple := range al {
    sum += len(tuple.StorageKeys)
  }
  return sum
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

// MarshalJSON marshals as JSON with a hash.
func (tx *Transaction) MarshalJSON() ([]byte, error) {
	var enc txJSON
  
	// Other fields are set conditionally depending on tx type.
	switch tx.Type {
	case LegacyTxType:
		enc.GasPrice = (*common.Big)(tx.GasPrice)
	case AccessListTxType:
		enc.ChainID = (*common.Big)(tx.ChainID)
		enc.AccessList = &tx.AccessList
		enc.GasPrice = (*common.Big)(tx.GasPrice)
	case DynamicFeeTxType:
		enc.ChainID = (*common.Big)(tx.ChainID)
		enc.AccessList = &tx.AccessList
		enc.MaxFeePerGas = (*common.Big)(tx.MaxFeePerGas)
		enc.MaxPriorityFeePerGas = (*common.Big)(tx.MaxPriorityFeePerGas)
	}

	enc.Type = common.Uint64(tx.Type)
	enc.Nonce = (*common.Uint64)(&tx.Nonce)
	enc.Gas = (*common.Uint64)(&tx.Gas)
  enc.Value = (*common.Big)(tx.Value)
  enc.Data = (*common.Bytes)(&tx.Data)
  enc.To = tx.To
  enc.V = (*common.Big)(tx.V)
  enc.R = (*common.Big)(tx.R)
  enc.S = (*common.Big)(tx.S)

	return json.Marshal(&enc)
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

// UnmarshalBinary decodes the canonical encoding of transactions.
// It supports legacy RLP transactions and EIP2718 typed transactions.
func (tx *Transaction) UnmarshalBinary(b []byte) error {
	if len(b) > 0 && b[0] > 0x7f {
		// It's a legacy transaction.
		var data LegacyTx
		err := rlp.DecodeBytes(b, &data)
		if err != nil {
			return err
		}
// 		tx = data
		return nil
	}

	// It's an EIP2718 typed transaction envelope.
	if len(b) <= 1 {
		return errShortTypedTx
	}
	switch b[0] {
	case AccessListTxType:
		var data AccessListTx
		err := rlp.DecodeBytes(b[1:], &data)

    if err != nil {
      return err
    }

//     tx = data
    return nil
	case DynamicFeeTxType:
		var data DynamicFeeTx
		err := rlp.DecodeBytes(b[1:], &data)

    if err != nil {
      return err
    }

//     tx = data
    return nil
	default:
		return ErrTxTypeNotSupported
	}
}

// MarshalBinary returns the canonical encoding of the transaction.
// For legacy transactions, it returns the RLP encoding. For EIP-2718 typed
// transactions, it returns the type and payload.
// func (tx *Transaction) MarshalBinary() ([]byte, error) {
//   switch tx.Type {
//   case AccessListTxType:
//     var buf bytes.Buffer
//     
//     data := &AccessListTx{tx.ChainID, tx.Nonce, tx.GasPrice, tx.Gas, tx.To, tx.Value, tx.Data, tx.AccessList, tx.V, tx.R, tx.S}
//     
//     buf.WriteByte(tx.Type)
//     err := rlp.Encode(buf, data)
//     
//     return buf.Bytes(), err
//   case DynamicFeeTxType:
//     var buf bytes.Buffer
//     
//     data := &DynamicFeeTx{tx.ChainID, tx.Nonce, tx.MaxPriorityFeePerGas, tx.MaxFeePerGas, tx.Gas, tx.To, tx.Value, tx.Data, tx.AccessList, tx.V, tx.R, tx.S}
//     
//     buf.WriteByte(tx.Type)
//     err := rlp.Encode(buf, data)
//     
//     return buf.Bytes(), err
//   default:
//     data := &LegacyTx{tx.Nonce, tx.GasPrice, tx.Gas, tx.To, tx.Value, tx.Data, tx.V, tx.R, tx.S}
//   
// 		return rlp.EncodeToBytes(data)
//   }
// }
// // // 
/*
// DecodeRLP implements rlp.Decoder
func (tx *Transaction) DecodeRLP(s *rlp.Stream) error {
	kind, size, err := s.Kind()
	switch {
	case err != nil:
		return err
	case kind == rlp.List:
		// It's a legacy transaction.
		var inner LegacyTx
		err := s.Decode(&inner)
		if err == nil {
			tx.setDecoded(&inner, rlp.ListSize(size))
		}
		return err
	default:
		// It's an EIP-2718 typed TX envelope.
		var b []byte
		if b, err = s.Bytes(); err != nil {
			return err
		}
		inner, err := tx.decodeTyped(b)
		if err == nil {
			tx.setDecoded(inner, uint64(len(b)))
		}
		return err
	}
}*/

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
    // It's an EIP-2718 typed TX envelope.
    buf := encodeBufferPool.Get().(*bytes.Buffer)
    defer encodeBufferPool.Put(buf)
    buf.Reset()
    buf.WriteByte(tx.Type)

    data := &AccessListTx{tx.ChainID, tx.Nonce, tx.GasPrice, tx.Gas, tx.To, tx.Value, tx.Data, tx.AccessList, tx.V, tx.R, tx.S}

    if err := rlp.Encode(buf, data); err != nil {
      return err
    }

    return rlp.Encode(w, buf.Bytes())
  case DynamicFeeTxType:
    // It's an EIP-2718 typed TX envelope.
    buf := encodeBufferPool.Get().(*bytes.Buffer)
    defer encodeBufferPool.Put(buf)
    buf.Reset()
    buf.WriteByte(tx.Type)

    data := &DynamicFeeTx{tx.ChainID, tx.Nonce, tx.MaxPriorityFeePerGas, tx.MaxFeePerGas, tx.Gas, tx.To, tx.Value, tx.Data, tx.AccessList, tx.V, tx.R, tx.S}

    if err := rlp.Encode(buf, data); err != nil {
      return err
    }

    return rlp.Encode(w, buf.Bytes())
  default: // LegacyTxType
    data := &LegacyTx{tx.Nonce, tx.GasPrice, tx.Gas, tx.To, tx.Value, tx.Data, tx.V, tx.R, tx.S}

    return rlp.Encode(w, data)
  }
}
