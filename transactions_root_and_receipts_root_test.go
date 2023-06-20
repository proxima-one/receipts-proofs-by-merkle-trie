package main

import (
	"encoding/hex"
	"encoding/json"
  "context"
	"fmt"
	"io/ioutil"
	"math/big"
	"os"
	"testing"
  "strings"
  "flag"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/ethereum/go-ethereum/core/types"
	"merkle-patrica-trie/rlp"
	"github.com/ethereum/go-ethereum/ethclient"
	"github.com/stretchr/testify/require"
  "github.com/joho/godotenv"
)

var blockNumber = flag.Int("blockNumber", 10467135, "The block number to test")

func GetTxSender(tx *types.Transaction) (common.Address, error) {
    signer := types.NewEIP155Signer(tx.ChainId())
    from, err := types.Sender(signer, tx)
    if err != nil {
        return common.Address{}, err
    }
    return from, nil
}

func TestTransactionsRootAndReceiptsRootAndProof(t *testing.T) {
  flag.Parse()
  
  err := godotenv.Load()
  require.NoError(t, err)

  client, err := ethclient.Dial("https://mainnet.infura.io/v3/"+os.Getenv("INFURA_API_KEY"))

  blockNumber := *blockNumber
  fmt.Println("blockNumber:", blockNumber)
  blockHeader, err := client.HeaderByNumber(context.Background(), big.NewInt(int64(blockNumber)))
  require.NoError(t, err)

  transactionsRootHex := blockHeader.TxHash.Hex()
  fmt.Println("transactionsRootHex:", transactionsRootHex)
  transactionsRootByte, err := hex.DecodeString(transactionsRootHex[2:])
//   fmt.Println("transactionsRootByte:", transactionsRootByte)

  block, err := client.BlockByNumber(context.Background(), big.NewInt(int64(blockNumber)))
  require.NoError(t, err)

  receiptsRootHex := block.ReceiptHash().Hex()
  fmt.Println("receiptsRootHex:", receiptsRootHex)
  receiptsRootByte, err := hex.DecodeString(receiptsRootHex[2:])

  transactions := make([]map[string]interface{}, 0)
  var receipts []*types.Receipt

  transactionsCount := len(block.Transactions())
  fmt.Println("transactionsCount:", transactionsCount)

  for i, tx := range block.Transactions() {
		txDict := make(map[string]interface{})

    sender, _ := GetTxSender(tx)
    v, r, s := tx.RawSignatureValues()

    var to *common.Address
    if tx.To() != nil {
        to = &common.Address{}
        *to = *tx.To()
    }

		txDict["blockHash"] = blockHeader.Hash().Hex()
		txDict["blockNumber"] = fmt.Sprintf("0x%x", blockNumber)
 		txDict["from"] = strings.ToLower(sender.Hex())
		txDict["gas"] = fmt.Sprintf("0x%x", tx.Gas())
		txDict["gasPrice"] = fmt.Sprintf("0x%x", tx.GasPrice())
		txDict["hash"] = tx.Hash().Hex()
		txDict["input"] = fmt.Sprintf("0x%x", tx.Data())
		txDict["nonce"] = fmt.Sprintf("0x%x", tx.Nonce())
 		txDict["r"] = fmt.Sprintf("0x%x", r)
 		txDict["s"] = fmt.Sprintf("0x%x", s)
		txDict["to"] = to
 		txDict["transactionIndex"] = fmt.Sprintf("0x%x", i)
 		txDict["v"] = fmt.Sprintf("0x%x", v.Uint64())
    txDict["value"] = fmt.Sprintf("0x%x", tx.Value())

		transactions = append(transactions, txDict)
    fmt.Println("Transaction hash:", tx.Hash().Hex())
    receipt, err := client.TransactionReceipt(context.Background(), tx.Hash())
//     fmt.Println("err:", err)
//     fmt.Println("receipt:", receipt)
    require.NoError(t, err)
		// fmt.Println("Logs:", receipt.Logs)
		for _, log := range receipt.Logs {
			fmt.Println("Address:", log.Address.Hex())
			fmt.Println("Data:", hexutil.Encode(log.Data))

			for _, topic := range log.Topics {
				fmt.Println("Topic:", topic.Hex())
			}
		}

//     fmt.Println("Transaction Receipt:", receipt)
    
    receipts = append(receipts, receipt)
	}
	
//  	fmt.Println(transactions[0])

  jsonBytes, err := json.MarshalIndent(transactions, "", "    ")
  require.NoError(t, err)
  
  fileName := fmt.Sprintf("transactions_from_block_%d.json", blockNumber)
  err = ioutil.WriteFile(fileName, []byte(jsonBytes), 0644)
  require.NoError(t, err)

	transactionsTrie := NewTrie()

 	txs := TransactionsJSONFromFile(t, fileName)

 	for i, tx := range txs {
// 	for i, tx := range block.Transactions() {
		// key is the encoding of the index as the unsigned integer type
		key, err := rlp.EncodeToBytes(uint(i))
		require.NoError(t, err)

		transaction := FromEthTransaction(tx)

		// value is the RLP encoding of a transaction
		rlp, err := transaction.GetRLP()
		require.NoError(t, err)

		transactionsTrie.Put(key, rlp)
	}
	
	receiptsTrie := NewTrie()

	for i, receipt := range receipts {
		// key is the encoding of the index as the unsigned integer type
		key, err := rlp.EncodeToBytes(uint(i))
		require.NoError(t, err)

		// value is the RLP encoding of a receipt
		rlp, err := rlp.EncodeToBytes(receipt)
		require.NoError(t, err)

		receiptsTrie.Put(key, rlp)
	}

	// the transaction root for block 10467135
	// https://api.etherscan.io/api?module=proxy&action=eth_getBlockByNumber&tag=0x9fb73f&boolean=true&apikey=YourApiKeyToken
// 	transactionsRoot, err := hex.DecodeString("bb345e208bda953c908027a45aa443d6cab6b8d2fd64e83ec52f1008ddeafa58")
//   fmt.Println("transactionsRoot:", transactionsRoot)

//   receiptsRoot, err := hex.DecodeString("494073bc875ed2e69edf96052b5910747f441cbaa9ead28a3594433006d1379d")
// 	require.NoError(t, err)

	t.Run("Merkle root hash should match with transactionsRoot", func(t *testing.T) {
		// transaction root should match with block transactionsRoot
		require.Equal(t, transactionsRootByte, transactionsTrie.Hash())
	})

	t.Run("Merkle root hash should match with receiptsRoot", func(t *testing.T) {
		// transaction root should match with block transactionsRoot
		require.Equal(t, receiptsRootByte, receiptsTrie.Hash())
	})

	t.Run("A Merkle proof for a certain transaction can be verified by the offical trie implementation", func(t *testing.T) {
		key, err := rlp.EncodeToBytes(uint(transactionsCount - 1))
		require.NoError(t, err)

		proof, found := transactionsTrie.Prove(key)
		require.Equal(t, true, found)

		txRLP, err := VerifyProof(transactionsRootByte, key, proof)
		require.NoError(t, err)

		// verify that if the verification passes, it returns the RLP encoded transaction
		rlp, err := FromEthTransaction(txs[transactionsCount - 1]).GetRLP()
		require.NoError(t, err)
		require.Equal(t, rlp, txRLP)
	})
}

func TransactionsReceiptsJSONFromFile(t *testing.T, fileName string) []*types.Receipt {
	jsonFile, err := os.Open(fileName)
	defer jsonFile.Close()
	require.NoError(t, err)
	byteValue, err := ioutil.ReadAll(jsonFile)
	require.NoError(t, err)
	var receipts []*types.Receipt
	json.Unmarshal(byteValue, &receipts)
	return receipts
}

func TransactionsJSONFromFile(t *testing.T, fileName string) []*types.Transaction {
	jsonFile, err := os.Open(fileName)
	defer jsonFile.Close()
	require.NoError(t, err)
	byteValue, err := ioutil.ReadAll(jsonFile)
	require.NoError(t, err)
	var txs []*types.Transaction
	json.Unmarshal(byteValue, &txs)
	return txs
}

// func FromEthTransactionReceipt(r *types.Receipt) *Receipt {
// 	return &Receipt{
// // 		Type:              r.Type,
// 		PostState:         r.PostState,
// 		Status:            r.Status,
// 		CumulativeGasUsed: r.CumulativeGasUsed,
// 		Bloom:             r.Bloom,
// 		Logs:              r.Logs,
// 		TxHash:            r.TxHash,
// 		ContractAddress:   r.ContractAddress,
// 		GasUsed:           r.GasUsed,
// //     EffectiveGasPrice: r.EffectiveGasPrice,
//     BlockHash:         r.BlockHash,
//     BlockNumber:       r.BlockNumber,
//     TransactionIndex:  r.TransactionIndex,
// 	}
// }

