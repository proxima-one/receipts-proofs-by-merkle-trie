package main

import (
  "context"
  "encoding/hex"
  "encoding/json"
  "flag"
  "fmt"
  "io/ioutil"
  "math/big"
  "os"
  "testing"

  "github.com/ethereum/go-ethereum/common"
  "github.com/ethereum/go-ethereum/common/hexutil"
  "github.com/ethereum/go-ethereum/core/types"
  "github.com/ethereum/go-ethereum/rlp"
  "github.com/ethereum/go-ethereum/rpc"
  "github.com/joho/godotenv"
  "github.com/stretchr/testify/require"
)

func TestRpcTransactionsRootAndReceiptsRootAndProof(t *testing.T) {
  flag.Parse()
  godotenv.Load()

  rpcClient, _ := rpc.Dial("https://mainnet.infura.io/v3/" + os.Getenv("INFURA_API_KEY"))

  defer rpcClient.Close()

  blockNumber := *blockNumber
  fmt.Println("BlockNumber:", blockNumber)

  var blockFromRpc map[string]interface{}
  rpcClient.CallContext(context.Background(), &blockFromRpc, "eth_getBlockByNumber", hexutil.EncodeBig(big.NewInt(int64(blockNumber))), true)

  fmt.Println("Timestamp:", blockFromRpc["timestamp"])
  fmt.Println("Size:", blockFromRpc["size"])
  fmt.Println("Hash:", blockFromRpc["hash"])
  fmt.Println("ParentHash:", blockFromRpc["parentHash"])
  fmt.Println("Nonce:", blockFromRpc["nonce"])
  fmt.Println("Difficulty:", blockFromRpc["difficulty"])
  fmt.Println("GasLimit:", blockFromRpc["gasLimit"])
  fmt.Println("GasUsed:", blockFromRpc["gasUsed"])
  fmt.Println("Miner:", blockFromRpc["miner"])
  fmt.Println("ExtraData:", blockFromRpc["extraData"])
  fmt.Println("LogsBloom:", blockFromRpc["logsBloom"])
  fmt.Println("MixHash:", blockFromRpc["mixHash"])
  fmt.Println("Sha3Uncles:", blockFromRpc["sha3Uncles"])
  fmt.Println("StateRoot:", blockFromRpc["stateRoot"])
  fmt.Println("ReceiptsRoot:", blockFromRpc["receiptsRoot"])
  fmt.Println("TransactionsRoot:", blockFromRpc["transactionsRoot"])
  fmt.Println("Uncles:", blockFromRpc["uncles"])

  rpcTransactions := blockFromRpc["transactions"].([]interface{})

  // Print transactions hashes
  transactionsHashes := []string{}
  for _, tx := range rpcTransactions {
    txData := tx.(map[string]interface{})
    transactionsHashes = append(transactionsHashes, txData["hash"].(string))
  }
  fmt.Println("TransactionsHashes:", transactionsHashes)

  transactions := make([]map[string]interface{}, 0)

  var receipts []*types.Receipt

  transactionsCount := len(rpcTransactions)
  fmt.Println("transactionsCount:", transactionsCount)

  for _, tx := range rpcTransactions {
    txDict := make(map[string]interface{})
    txData := tx.(map[string]interface{})

//     fmt.Println("Transaction hash:", txData["hash"].(string))

    txDict["gas"] = txData["gas"]
    txDict["gasPrice"] = txData["gasPrice"]
    txDict["input"] = txData["input"]
    txDict["nonce"] = txData["nonce"]
    txDict["v"] = txData["v"]
    txDict["r"] = txData["r"]
    txDict["s"] = txData["s"]
    txDict["to"] = txData["to"]
    txDict["value"] = txData["value"]

    transactions = append(transactions, txDict)

    var receipt *types.Receipt
    rpcClient.CallContext(context.Background(), &receipt, "eth_getTransactionReceipt", common.HexToHash(txData["hash"].(string)))

//     fmt.Println("Transaction Receipt:", receipt)

    receipts = append(receipts, receipt)
  }

  //    fmt.Println(transactions[0])

  jsonBytes, err := json.MarshalIndent(transactions, "", "    ")
  require.NoError(t, err)

  fileName := fmt.Sprintf("transactions_from_block_%d.json", blockNumber)
  err = ioutil.WriteFile(fileName, []byte(jsonBytes), 0644)
  require.NoError(t, err)

  transactionsTrie := NewTrie()

  txsFromJson := TransactionsJSONFromFile(t, fileName)

  for i, tx := range txsFromJson {
    //   for i, tx := range block.Transactions() {
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

  transactionsRootByte, _ := hex.DecodeString(blockFromRpc["transactionsRoot"].(string)[2:])
  //   fmt.Println("transactionsRootByte:", transactionsRootByte)

  t.Run("Merkle root hash should match with transactionsRoot", func(t *testing.T) {
    // transaction root should match with block transactionsRoot
    require.Equal(t, transactionsRootByte, transactionsTrie.Hash())
  })

  receiptsRootByte, err := hex.DecodeString(blockFromRpc["receiptsRoot"].(string)[2:])

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
    rlp, err := FromEthTransaction(txsFromJson[transactionsCount-1]).GetRLP()
    require.NoError(t, err)
    require.Equal(t, rlp, txRLP)
  })
}
