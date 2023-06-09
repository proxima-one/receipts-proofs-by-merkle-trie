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

  "merkle-patrica-trie/common"
  "merkle-patrica-trie/rlp"
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
  rpcClient.CallContext(context.Background(), &blockFromRpc, "eth_getBlockByNumber", common.EncodeBig(big.NewInt(int64(blockNumber))), true)

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
//   transactionsHashes := []string{}
//   for _, tx := range rpcTransactions {
//     txData := tx.(map[string]interface{})
//     transactionsHashes = append(transactionsHashes, txData["hash"].(string))
//   }
//   fmt.Println("TransactionsHashes:", transactionsHashes)

  transactions := make([]map[string]interface{}, 0)
  receipts := make([]map[string]interface{}, 0)

  transactionsCount := len(rpcTransactions)
  fmt.Println("transactionsCount:", transactionsCount)

  for _, tx := range rpcTransactions {
    minimizedTx := make(map[string]interface{})
    txData := tx.(map[string]interface{})

//     fmt.Println("Transaction hash:", txData["hash"].(string))
//     fmt.Println("txData:", txData)

    minimizedTx["type"] = txData["type"]
    if (txData["chainId"] != nil) {
      minimizedTx["chainId"] = txData["chainId"]
    }
    minimizedTx["nonce"] = txData["nonce"]
    if (txData["maxPriorityFeePerGas"] != nil) {
      minimizedTx["maxPriorityFeePerGas"] = txData["maxPriorityFeePerGas"]
    }
    if (txData["maxFeePerGas"] != nil) {
      minimizedTx["maxFeePerGas"] = txData["maxFeePerGas"]
    }
    if (txData["gasPrice"] != nil) {
      minimizedTx["gasPrice"] = txData["gasPrice"]
    }
    minimizedTx["gas"] = txData["gas"]
    minimizedTx["input"] = txData["input"]
    minimizedTx["v"] = txData["v"]
    minimizedTx["r"] = txData["r"]
    minimizedTx["s"] = txData["s"]
    minimizedTx["to"] = txData["to"]
    minimizedTx["value"] = txData["value"]
    if (txData["accessList"] != nil) {
      minimizedTx["accessList"] = txData["accessList"]
    }

//     fmt.Println("Minimized Transaction:", minimizedTx)

    transactions = append(transactions, minimizedTx)

    var receipt map[string]interface{}
    rpcClient.CallContext(context.Background(), &receipt, "eth_getTransactionReceipt", common.HexToHash(txData["hash"].(string)))

//     fmt.Println("Transaction Receipt:", receipt)

    minimizedReceipt := make(map[string]interface{})

    minimizedReceipt["type"] = receipt["type"]
    minimizedReceipt["status"] = receipt["status"]
    minimizedReceipt["cumulativeGasUsed"] = receipt["cumulativeGasUsed"]
    minimizedReceipt["logsBloom"] = receipt["logsBloom"]
    minimizedReceipt["gasUsed"] = receipt["gasUsed"]

    minimizedlogs := make([]map[string]interface{}, 0)
    for _, log := range receipt["logs"].([]interface{}) {
      minimizedLog := make(map[string]interface{})
      logData := log.(map[string]interface{})

      minimizedLog["address"] = logData["address"]
      minimizedLog["data"] = logData["data"]
      minimizedLog["topics"] = logData["topics"]

      minimizedlogs = append(minimizedlogs, minimizedLog)
    }
    minimizedReceipt["logs"] = minimizedlogs

//     fmt.Println("Minimized Receipt:", minimizedReceipt)

    receipts = append(receipts, minimizedReceipt)
  }

  jsonBytes, _ := json.MarshalIndent(transactions, "", "    ")

  fileName := fmt.Sprintf("transactions_from_block_%d.json", blockNumber)
  ioutil.WriteFile(fileName, []byte(jsonBytes), 0644)

  transactionsTrie := NewTrie()

  txsFromJson := TransactionsFromJSON(t, fileName)

  jsonBytes, _ = json.MarshalIndent(txsFromJson, "", "    ")

  fileName = fmt.Sprintf("transactions2_from_block_%d.json", blockNumber)
  ioutil.WriteFile(fileName, []byte(jsonBytes), 0644)

  for i, tx := range txsFromJson {
    // key is the encoding of the index as the unsigned integer type
    key, _ := rlp.EncodeToBytes(uint(i))

    // value is the RLP encoding of a transaction
    rlp, _ := rlp.EncodeToBytes(tx)

    transactionsTrie.Put(key, rlp)
  }

  transactionsRootByte, _ := hex.DecodeString(blockFromRpc["transactionsRoot"].(string)[2:])
  //   fmt.Println("transactionsRootByte:", transactionsRootByte)

  t.Run("Merkle root hash should match with transactionsRoot", func(t *testing.T) {
    // transaction root should match with block transactionsRoot
    require.Equal(t, transactionsRootByte, transactionsTrie.Hash())
  })

  t.Run("A Merkle proof for a certain transaction can be verified by the offical trie implementation", func(t *testing.T) {
    key, err := rlp.EncodeToBytes(uint(transactionsCount - 1))
    require.NoError(t, err)

    proof, found := transactionsTrie.Prove(key)
    require.Equal(t, true, found)

    txRLP, err := VerifyProof(transactionsRootByte, key, proof)
    require.NoError(t, err)

    // verify that if the verification passes, it returns the RLP encoded transaction
    rlp, err := rlp.EncodeToBytes(txsFromJson[transactionsCount-1])
    require.NoError(t, err)
    require.Equal(t, rlp, txRLP)
  })

  jsonBytes, _ = json.MarshalIndent(receipts, "", "    ")

  fileName = fmt.Sprintf("transactions_receipts_from_block_%d.json", blockNumber)
  ioutil.WriteFile(fileName, []byte(jsonBytes), 0644)

  receiptsTrie := NewTrie()

  receiptsFromJson := TransactionsReceiptsFromJSON(t, fileName)

  jsonBytes, _ = json.MarshalIndent(receiptsFromJson, "", "    ")

  fileName = fmt.Sprintf("transactions_receipts2_from_block_%d.json", blockNumber)
  ioutil.WriteFile(fileName, []byte(jsonBytes), 0644)

  for i, receipt := range receiptsFromJson {
    // key is the encoding of the index as the unsigned integer type
    key, _ := rlp.EncodeToBytes(uint(i))

    // value is the RLP encoding of a receipt
    rlp, _ := rlp.EncodeToBytes(receipt)

    receiptsTrie.Put(key, rlp)
  }
  
  receiptsRootByte, _ := hex.DecodeString(blockFromRpc["receiptsRoot"].(string)[2:])

  t.Run("Merkle root hash should match with receiptsRoot", func(t *testing.T) {
    // transaction root should match with block transactionsRoot
    require.Equal(t, receiptsRootByte, receiptsTrie.Hash())
  })

  t.Run("A Merkle proof for a certain transaction receipt can be verified by the offical trie implementation", func(t *testing.T) {
    key, err := rlp.EncodeToBytes(uint(transactionsCount - 1))
    require.NoError(t, err)

    proof, found := receiptsTrie.Prove(key)
    require.Equal(t, true, found)

    receiptRLP, err := VerifyProof(receiptsRootByte, key, proof)
    require.NoError(t, err)

    // verify that if the verification passes, it returns the RLP encoded transaction receipt
    rlp, err := rlp.EncodeToBytes(receiptsFromJson[transactionsCount-1])
    require.NoError(t, err)
    require.Equal(t, rlp, receiptRLP)
  })
}

func TransactionsFromJSON(t *testing.T, fileName string) []*Transaction {
  jsonFile, err := os.Open(fileName)
  defer jsonFile.Close()
  require.NoError(t, err)
  byteValue, err := ioutil.ReadAll(jsonFile)
  require.NoError(t, err)
  var txs []*Transaction
  json.Unmarshal(byteValue, &txs)
  return txs
}

func TransactionsReceiptsFromJSON(t *testing.T, fileName string) []*Receipt {
  jsonFile, err := os.Open(fileName)
  defer jsonFile.Close()
  require.NoError(t, err)
  byteValue, err := ioutil.ReadAll(jsonFile)
  require.NoError(t, err)
  var receipts []*Receipt
  json.Unmarshal(byteValue, &receipts)
  return receipts
}
