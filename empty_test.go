package main

import (
	"testing"

	"merkle-patrica-trie/rlp"
	"github.com/stretchr/testify/require"
  "fmt"
)

func TestEmptyNodeHash(t *testing.T) {
	emptyRLP, err := rlp.EncodeToBytes(EmptyNodeRaw)
//   fmt.Println("emptyRLP:", emptyRLP)
	require.NoError(t, err)
	require.Equal(t, EmptyNodeHash, Keccak256(emptyRLP))
}
