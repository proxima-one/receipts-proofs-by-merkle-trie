fn main() {

}

extern crate tiny_keccak;
use tiny_keccak::Keccak;
use tiny_keccak::Hasher;
use hex::FromHex;
use rlp::Rlp;

// nodes.go

// type Node interface {
//   Hash() []byte // common.Hash
//   Raw() []interface{}
// }
pub trait Node {
    fn hash(&self) -> Vec<u8>; 
    fn raw(&self) -> Vec<Vec<u8>>;
    fn is_empty(&self) -> bool;
}

// func Hash(node Node) []byte {
//   if IsEmptyNode(node) {
//     return EmptyNodeHash
//   }
//   return node.Hash()
// }
fn hash(node: &impl Node) -> Vec<u8> {
    if is_empty_node(node) {
        return EMPTY_NODE_HASH.to_vec()
    }
    node.hash()
}

// func Serialize(node Node) []byte {
//   var raw interface{}
// 
//   if IsEmptyNode(node) {
//     raw = EmptyNodeRaw
//   } else {
//     raw = node.Raw()
//   }
// 
//   rlp, err := rlp.EncodeToBytes(raw)
//   if err != nil {
//     panic(err)
//   }
// 
//   return rlp
// }
fn serialize(node: &impl Node) -> Vec<u8> {
    if is_empty_node(node) {
        return EMPTY_NODE_HASH.to_vec()
    } else {
        let raw = node.raw();
        return Rlp::new(&raw[0]).as_raw().to_vec()
    };
}

// empty.go

extern crate hex;

// EmptyNodeHash, _ = hex.DecodeString("56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421")
static EMPTY_NODE_HASH: [u8; 32] = [
	0x56, 0xe8, 0x1f, 0x17, 0x1b, 0xcc, 0x55, 0xa6, 0xff, 0x83, 0x45, 0xe6, 0x92, 0xc0, 0xf8, 0x6e,
	0x5b, 0x48, 0xe0, 0x1b, 0x99, 0x6c, 0xad, 0xc0, 0x01, 0x62, 0x2f, 0xb5, 0xe3, 0x63, 0xb4, 0x21,
];

// func IsEmptyNode(node Node) bool {
//   return node == nil
// }
// pub fn is_empty_node(node: &Option<Vec<u8>>) -> bool {
pub fn is_empty_node<T: Node>(node: &T) -> bool {
    node.is_empty()
}

fn keccak256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak::v256();
    let mut hash = [0u8; 32];
    hasher.update(data);
    hasher.finalize(&mut hash);
    hash
}

// nibbles.go

// type Nibble byte
#[derive(Debug, PartialEq, Clone)]
pub struct Nibble(u8);

impl Nibble {
    fn to_usize(&self) -> usize {
        self.0 as usize
    }

    // func IsNibble(nibble byte) bool {
    //   n := int(nibble)
    //   // 0-9 && a-f
    //   return n >= 0 && n < 16
    // }
    fn is_nibble(nibble: u8) -> bool {
        let n = nibble as usize;
        n >= 0 && n < 16
    }

    // func FromNibbleByte(n byte) (Nibble, error) {
    //   if !IsNibble(n) {
    //     return 0, fmt.Errorf("non-nibble byte: %v", n)
    //   }
    //   return Nibble(n), nil
    // }
    fn from_nibble_byte(n: u8) -> Result<Nibble, &'static str> {
        if !Nibble::is_nibble(n) {
            return Err("Non-nibble byte");
        }
        Ok(Nibble(n))
    }

    // // nibbles contain one nibble per byte
    // func FromNibbleBytes(nibbles []byte) ([]Nibble, error) {
    //   ns := make([]Nibble, 0, len(nibbles))
    //   for _, n := range nibbles {
    //     nibble, err := FromNibbleByte(n)
    //     if err != nil {
    //       return nil, fmt.Errorf("contains non-nibble byte: %w", err)
    //     }
    //     ns = append(ns, nibble)
    //   }
    //   return ns, nil
    // }
    fn from_nibble_bytes(nibbles: Vec<u8>) -> Result<Vec<Nibble>, &'static str> {
        let mut ns = Vec::with_capacity(nibbles.len());
        for n in nibbles {
            let nibble = Nibble::from_nibble_byte(n)?;
            ns.push(nibble);
        }
        Ok(ns)
    }

    // func FromByte(b byte) []Nibble {
    //   return []Nibble{
    //     Nibble(byte(b >> 4)),
    //     Nibble(byte(b % 16)),
    //   }
    // }
    fn from_byte(b: u8) -> Vec<Nibble> {
        vec![
            Nibble((b >> 4) as u8),
            Nibble((b % 16) as u8),
        ]
    }

    // func FromBytes(bs []byte) []Nibble {
    //   ns := make([]Nibble, 0, len(bs)*2)
    //   for _, b := range bs {
    //     ns = append(ns, FromByte(b)...)
    //   }
    //   return ns
    // }
    fn from_bytes(bs: Vec<u8>) -> Vec<Nibble> {
        let mut ns = Vec::with_capacity(bs.len() * 2);
        for b in bs {
            ns.extend(Nibble::from_byte(b));
        }
        ns
    }

    // func FromString(s string) []Nibble {
    //   return FromBytes([]byte(s))
    // }
    fn from_string(s: String) -> Vec<Nibble> {
        Nibble::from_bytes(s.into_bytes())
    }

    // // ToPrefixed add nibble prefix to a slice of nibbles to make its length even
    // // the prefix indicts whether a node is a leaf node.
    // func ToPrefixed(ns []Nibble, isLeafNode bool) []Nibble {
    //   // create prefix
    //   var prefixBytes []Nibble
    //   // odd number of nibbles
    //   if len(ns)%2 > 0 {
    //     prefixBytes = []Nibble{1}
    //   } else {
    //     // even number of nibbles
    //     prefixBytes = []Nibble{0, 0}
    //   }
    // 
    //   // append prefix to all nibble bytes
    //   prefixed := make([]Nibble, 0, len(prefixBytes)+len(ns))
    //   prefixed = append(prefixed, prefixBytes...)
    //   prefixed = append(prefixed, ns...)
    // 
    //   // update prefix if is leaf node
    //   if isLeafNode {
    //     prefixed[0] += 2
    //   }
    // 
    //   return prefixed
    // }
    fn to_prefixed(ns: Vec<Nibble>, is_leaf_node: bool) -> Vec<Nibble> {
        let mut prefix_bytes = if ns.len() % 2 > 0 {
            vec![Nibble(1)]
        } else {
            vec![Nibble(0), Nibble(0)]
        };

        let mut prefixed = Vec::with_capacity(prefix_bytes.len() + ns.len());
        prefixed.append(&mut prefix_bytes);
        prefixed.extend(ns);

        if is_leaf_node {
            prefixed[0].0 += 2;
        }

        prefixed
    }

    // // ToBytes converts a slice of nibbles to a byte slice
    // // assuming the nibble slice has even number of nibbles.
    // func ToBytes(ns []Nibble) []byte {
    //   buf := make([]byte, 0, len(ns)/2)
    // 
    //   for i := 0; i < len(ns); i += 2 {
    //     b := byte(ns[i]<<4) + byte(ns[i+1])
    //     buf = append(buf, b)
    //   }
    // 
    //   return buf
    // }
    fn to_bytes(ns: Vec<Nibble>) -> Vec<u8> {
        let mut buf = Vec::with_capacity(ns.len() / 2);

        for ns_chunk in ns.chunks_exact(2) {
            let b = (ns_chunk[0].0 << 4) + ns_chunk[1].0;
            buf.push(b);
        }

        buf
    }

    // // [0,1,2,3], [0,1,2] => 3
    // // [0,1,2,3], [0,1,2,3] => 4
    // // [0,1,2,3], [0,1,2,3,4] => 4
    // func PrefixMatchedLen(node1 []Nibble, node2 []Nibble) int {
    //   matched := 0
    //   for i := 0; i < len(node1) && i < len(node2); i++ {
    //     n1, n2 := node1[i], node2[i]
    //     if n1 == n2 {
    //       matched++
    //     } else {
    //       break
    //     }
    //   }
    // 
    //   return matched
    // }
    fn prefix_matched_len(node1: &[Nibble], node2: &[Nibble]) -> usize {
        node1.iter().zip(node2.iter()).take_while(|&(n1, n2)| n1 == n2).count()
    }
}

fn encode_nibbles(nibbles: &Vec<Nibble>) -> String {
    let mut result = String::with_capacity(nibbles.len() * 2);
    for nibble in nibbles {
        let byte = nibble.0;
        let hex_chars = hex::encode_upper(&[byte]);
        result.push_str(&hex_chars);
    }
    result
}

// leaf.go

// type LeafNode struct {
//   Path  []Nibble
//   Value []byte
// }
pub struct LeafNode {
    path: Vec<Nibble>,
    value: Vec<u8>,
}

impl Node for LeafNode {
    fn hash(&self) -> Vec<u8> {
        self.hash()
    }

    fn raw(&self) -> Vec<Vec<u8>> {
        self.raw()
    }

    fn is_empty(&self) -> bool {
        self.value.is_empty()
    }
}

impl LeafNode {
    // func NewLeafNodeFromNibbleBytes(nibbles []byte, value []byte) (*LeafNode, error) {
    //   ns, err := FromNibbleBytes(nibbles)
    //   if err != nil {
    //     return nil, fmt.Errorf("could not leaf node from nibbles: %w", err)
    //   }
    // 
    //   return NewLeafNodeFromNibbles(ns, value), nil
    // }
    pub fn new_from_nibble_bytes(nibbles: &[u8], value: &[u8]) -> Result<LeafNode, &'static str> {
        let ns = Nibble::from_nibble_bytes(nibbles.to_vec())?;
        Ok(LeafNode {
            path: ns,
            value: value.to_vec(),
        })
    }

    // func NewLeafNodeFromNibbles(nibbles []Nibble, value []byte) *LeafNode {
    //   return &LeafNode{
    //     Path:  nibbles,
    //     Value: value,
    //   }
    // }
    pub fn new_leaf_node_from_nibbles(nibbles: Vec<Nibble>, value: Vec<u8>) -> LeafNode {
        LeafNode {
            path: nibbles,
            value: value,
        }
    }

    // func NewLeafNodeFromKeyValue(key, value string) *LeafNode {
    //   return NewLeafNodeFromBytes([]byte(key), []byte(value))
    // }
    pub fn new_from_key_value(key: &str, value: &str) -> LeafNode {
        LeafNode::new_from_bytes(key.as_bytes(), value.as_bytes())
    }

    // func NewLeafNodeFromBytes(key, value []byte) *LeafNode {
    //   return NewLeafNodeFromNibbles(FromBytes(key), value)
    // }
    pub fn new_from_bytes(key: &[u8], value: &[u8]) -> LeafNode {
        LeafNode::new_leaf_node_from_nibbles(Nibble::from_bytes(key.to_vec()), value.to_vec())
    }

    // func (l LeafNode) Hash() []byte {
    //   return crypto.Keccak256(l.Serialize())
    // }
    pub fn hash(&self) -> Vec<u8> {
        keccak256(&self.serialize()).to_vec()
    }

    // func (l LeafNode) Raw() []interface{} {
    //   path := ToBytes(ToPrefixed(l.Path, true))
    //   raw := []interface{}{path, l.Value}
    //   return raw
    // }
    pub fn raw(&self) -> Vec<Vec<u8>> {
        let path_u8: Vec<u8> = Nibble::to_bytes(Nibble::to_prefixed(self.path.clone(), true));
        let value_any: Vec<u8> = self.value.clone();
        vec![path_u8, value_any]
    }

    // func (l LeafNode) Serialize() []byte {
    //   return Serialize(l)
    // }
    pub fn serialize(&self) -> Vec<u8> {
        serialize(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::vec::Vec;

    // func TestEmptyNodeHash(t *testing.T) {
    //   emptyRLP, err := rlp.EncodeToBytes(EmptyNodeRaw)
    //   require.NoError(t, err)
    //   require.Equal(t, EmptyNodeHash, Keccak256(emptyRLP))
    // }
    #[test]
    fn test_empty_node_hash() {
        let empty_node_hash: [u8; 32] = FromHex::from_hex("56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421").unwrap();
        assert_eq!(empty_node_hash, EMPTY_NODE_HASH);

        let empty_rlp = rlp::NULL_RLP.to_vec();
//         println!("empty_rlp: {:?}", empty_rlp);
        assert_eq!(empty_rlp, [0x80]);

        assert_eq!(EMPTY_NODE_HASH, keccak256(&empty_rlp));
    }

    // func TestIsNibble(t *testing.T) {
    //   for i := 0; i < 20; i++ {
    //     isNibble := i >= 0 && i < 16
    //     require.Equal(t, isNibble, IsNibble(byte(i)), i)
    //   }
    // }
    #[test]
    fn test_is_nibble() {
        for i in 0..20 {
            let is_nibble = i < 16;
            assert_eq!(is_nibble, Nibble::is_nibble(i as u8), "{}", i);
        }
    }

    // func TestToPrefixed(t *testing.T) {
    //   cases := []struct {
    //     ns         []Nibble
    //     isLeafNode bool
    //     expected   []Nibble
    //   }{
    //     {[]Nibble{1}, false, []Nibble{1, 1},},
    //     {[]Nibble{1, 2}, false, []Nibble{0, 0, 1, 2},},
    //     {[]Nibble{1}, true, []Nibble{3, 1},},
    //     {[]Nibble{1, 2}, true, []Nibble{2, 0, 1, 2},},
    //     {[]Nibble{5, 0, 6}, true, []Nibble{3, 5, 0, 6},},
    //     {[]Nibble{14, 3}, false, []Nibble{0, 0, 14, 3},},
    //     {[]Nibble{9, 3, 6, 5}, true, []Nibble{2, 0, 9, 3, 6, 5},},
    //     {[]Nibble{1, 3, 3, 5}, true, []Nibble{2, 0, 1, 3, 3, 5},},
    //     {[]Nibble{7}, true, []Nibble{3, 7},},
    //   }
    // 
    //   for _, c := range cases {
    //     require.Equal(t,
    //       c.expected,
    //       ToPrefixed(c.ns, c.isLeafNode))
    //   }
    // }
    #[test]
    fn test_to_prefixed() {
        let cases = vec![
            (vec![Nibble(1)], false, vec![Nibble(1), Nibble(1)]),
            (vec![Nibble(1), Nibble(2)], false, vec![Nibble(0), Nibble(0), Nibble(1), Nibble(2)]),
            (vec![Nibble(1)], true, vec![Nibble(3), Nibble(1)]),
            (vec![Nibble(1), Nibble(2)], true, vec![Nibble(2), Nibble(0), Nibble(1), Nibble(2)]),
            (vec![Nibble(5), Nibble(0), Nibble(6)], true, vec![Nibble(3), Nibble(5), Nibble(0), Nibble(6)]),
            (vec![Nibble(14), Nibble(3)], false, vec![Nibble(0), Nibble(0), Nibble(14), Nibble(3)]),
            (vec![Nibble(9), Nibble(3), Nibble(6), Nibble(5)], true, vec![Nibble(2), Nibble(0), Nibble(9), Nibble(3), Nibble(6), Nibble(5)]),
            (vec![Nibble(1), Nibble(3), Nibble(3), Nibble(5)], true, vec![Nibble(2), Nibble(0), Nibble(1), Nibble(3), Nibble(3), Nibble(5)]),
            (vec![Nibble(7)], true, vec![Nibble(3), Nibble(7)]),
        ];

        for (ns, is_leaf_node, expected) in cases {
            assert_eq!(expected, Nibble::to_prefixed(ns, is_leaf_node));
        }
    }

    // func TestFromBytes(t *testing.T) {
    //   // [1, 100] -> ['0x01', '0x64']
    //   require.Equal(t, []Nibble{0, 1, 6, 4}, FromBytes([]byte{1, 100}))
    // }
    #[test]
    fn test_from_bytes() {
        // [1, 100] -> ['0x01', '0x64']
        assert_eq!(
            vec![Nibble(0), Nibble(1), Nibble(6), Nibble(4)],
            Nibble::from_bytes((&[1, 100].to_vec()).to_vec())
        );
    }

    // func TestToBytes(t *testing.T) {
    //   bytes := []byte{0, 1, 2, 3}
    //   require.Equal(t, bytes, ToBytes(FromBytes(bytes)))
    // }
    #[test]
    fn test_to_bytes() {
        let bytes = &[0, 1, 2, 3];
        assert_eq!(bytes.to_vec(), Nibble::to_bytes(Nibble::from_bytes(bytes.to_vec())));
    }

    // func TestPrefixMatchedLen(t *testing.T) {
    //   require.Equal(t, 3, PrefixMatchedLen([]Nibble{0, 1, 2, 3}, []Nibble{0, 1, 2}))
    //   require.Equal(t, 4, PrefixMatchedLen([]Nibble{0, 1, 2, 3}, []Nibble{0, 1, 2, 3}))
    //   require.Equal(t, 4, PrefixMatchedLen([]Nibble{0, 1, 2, 3}, []Nibble{0, 1, 2, 3, 4}))
    // }
    #[test]
    fn test_prefix_matched_len() {
        assert_eq!(3, Nibble::prefix_matched_len(&[Nibble(0), Nibble(1), Nibble(2), Nibble(3)], &[Nibble(0), Nibble(1), Nibble(2)]));
        assert_eq!(4, Nibble::prefix_matched_len(&[Nibble(0), Nibble(1), Nibble(2), Nibble(3)], &[Nibble(0), Nibble(1), Nibble(2), Nibble(3)]));
        assert_eq!(4, Nibble::prefix_matched_len(&[Nibble(0), Nibble(1), Nibble(2), Nibble(3)], &[Nibble(0), Nibble(1), Nibble(2), Nibble(3), Nibble(4)]));
    }

    // func printEachCalculationSteps(key, value []byte, isLeaf bool) map[string]string {
    //   hexs := make(map[string]string)
    //   hexs["key in nibbles"] = fmt.Sprintf("%x", FromBytes(key))
    //   hexs["key in nibbles, and prefixed"] = fmt.Sprintf("%x", ToPrefixed(FromBytes(key), isLeaf))
    //   hexs["key in nibbles, and prefixed, and convert back to buffer"] =
    //     fmt.Sprintf("%x", ToBytes(ToPrefixed(FromBytes(key), isLeaf)))
    //   beforeRLP := [][]byte{ToBytes(ToPrefixed(FromBytes(key), isLeaf)), value}
    //   hexs["beforeRLP"] = fmt.Sprintf("%x", beforeRLP)
    //   afterRLP, err := rlp.EncodeToBytes(beforeRLP)
    //   if err != nil {
    //     panic(err)
    //   }
    //   hexs["afterRLP"] = fmt.Sprintf("%x", afterRLP)
    //   hexs["hash"] = fmt.Sprintf("%x", crypto.Keccak256(afterRLP))
    //   return hexs
    // }
    fn print_each_calculation_steps(key: &[u8], value: &[u8], is_leaf: bool) -> Vec<(String, String)> {
        let mut hexs: Vec<(String, String)> = Vec::new();
        hexs.push(("key in nibbles".to_owned(), encode_nibbles(&Nibble::from_bytes(key.to_vec()))));
        hexs.push(("key in nibbles, and prefixed".to_owned(), encode_nibbles(&Nibble::to_prefixed(Nibble::from_bytes(key.to_vec()), is_leaf))));
        hexs.push(("key in nibbles, and prefixed, and convert back to buffer".to_owned(), hex::encode(Nibble::to_bytes(Nibble::to_prefixed(Nibble::from_bytes(key.to_vec()), is_leaf)))));
        let before_rlp = vec![Nibble::to_bytes(Nibble::to_prefixed(Nibble::from_bytes(key.to_vec()), is_leaf)), value.to_vec()];
        hexs.push(("beforeRLP".to_owned(), hex::encode(&before_rlp[1])));
        let after_rlp = Rlp::new(&before_rlp[1]).as_raw();
        hexs.push(("afterRLP".to_owned(), hex::encode(&after_rlp)));
        hexs.push(("hash".to_owned(), hex::encode(keccak256(&after_rlp))));
        hexs
    }

    // func TestLeafHash(t *testing.T) {
    //   require.Equal(t, "01020304", fmt.Sprintf("%x", []byte{1, 2, 3, 4}))
    //   require.Equal(t, "76657262", fmt.Sprintf("%x", []byte("verb")))
    // 
    //   // "buffer to nibbles
    //   require.Equal(t, "0001000200030004", fmt.Sprintf("%x", FromBytes([]byte{1, 2, 3, 4})))
    // 
    //   // ToPrefixed
    //   require.Equal(t, "02000001000200030004", fmt.Sprintf("%x", ToPrefixed(FromBytes([]byte{1, 2, 3, 4}), true)))
    // 
    //   // ToBuffer
    //   require.Equal(t, "2001020304", fmt.Sprintf("%x", ToBytes(ToPrefixed(FromBytes([]byte{1, 2, 3, 4}), true))))
    // 
    //   require.Equal(t, "636f696e", fmt.Sprintf("%x", []byte("coin")))
    // }
    #[test]
    fn test_leaf_hash() {
        assert_eq!(hex::encode(&[1, 2, 3, 4]), "01020304");
        assert_eq!(hex::encode(b"verb"), "76657262");

        // "buffer to nibbles
        assert_eq!(
            encode_nibbles(&Nibble::from_bytes([1, 2, 3, 4].to_vec())),
            "0001000200030004"
        );

        // ToPrefixed
        assert_eq!(
            encode_nibbles(&Nibble::to_prefixed(Nibble::from_bytes([1, 2, 3, 4].to_vec()), true)),
            "02000001000200030004"
        );

        // ToBuffer
        assert_eq!(
            hex::encode(Nibble::to_bytes(Nibble::to_prefixed(Nibble::from_bytes([1, 2, 3, 4].to_vec()), true))),
            "2001020304"
        );

        assert_eq!(hex::encode(b"coin"), "636f696e");
    }

    // func Test3Nibbles(t *testing.T) {
    //   key, value := []byte{5, 0, 6}, []byte("coin")
    //   hexs := printEachCalculationSteps(key, value, true)
    //   fmt.Printf("key_hex: %x\n", key)
    //   fmt.Printf("value_hex: %x\n", value)
    //   fmt.Printf("key in nibbles: %s\n", hexs["key in nibbles"])
    //   fmt.Printf("key in nibbles, and prefixed: %s\n", hexs["key in nibbles, and prefixed"])
    //   fmt.Printf("key in nibbles, and prefixed, and convert back to buffer: %s\n",
    //     hexs["key in nibbles, and prefixed, and convert back to buffer"])
    //   fmt.Printf("beforeRLP: %s\n", hexs["beforeRLP"])
    //   fmt.Printf("afterRLP: %s\n", hexs["afterRLP"])
    //   fmt.Printf("hash: %s\n", hexs["hash"])
    //   require.Equal(t, "c5442690f038fcc0b8b8949b4f5149db8c0bee917be6355dc2db1855e9675700",
    //     hexs["hash"])
    // }
    #[test]
    fn test_3_nibbles() {
        let key = &[5, 0, 6];
        let value = b"coin";
        let hexs = print_each_calculation_steps(key, value, true);
        println!("key_hex: {:?}", key);
        println!("value_hex: {:?}", value);
        println!("key in nibbles: {:?}", hexs[0].1);
        println!("key in nibbles, and prefixed: {:?}", hexs[1].1);
        println!("key in nibbles, and prefixed, and convert back to buffer: {:?}", hexs[2].1);
        println!("beforeRLP: {:?}", hexs[3].1);
        println!("afterRLP: {:?}", hexs[4].1);
        println!("hash: {:?}", hexs[5].1);
        assert_eq!(
            hexs[5].1,
            "c5442690f038fcc0b8b8949b4f5149db8c0bee917be6355dc2db1855e9675700"
        );
    }

    // func TestLeafNode(t *testing.T) {
    //   nibbles, value := []byte{1, 2, 3, 4}, []byte("verb")
    //   l := NewLeafNodeFromBytes(nibbles, value)
    //   require.Equal(t, "2bafd1eef58e8707569b7c70eb2f91683136910606ba7e31d07572b8b67bf5c6", fmt.Sprintf("%x", l.Hash()))
    // }
    #[test]
    fn test_leaf_node() {
        let nibbles = &[1, 2, 3, 4];
        let value = b"verb";
        let l = LeafNode::new_from_bytes(nibbles, value);
        assert_eq!(
            hex::encode(l.hash()),
            "2bafd1eef58e8707569b7c70eb2f91683136910606ba7e31d07572b8b67bf5c6"
        );
    }

    // func TestLeafNode2(t *testing.T) {
    //   // t.Skip()
    //   nibbles, value := []byte{5, 0, 6}, []byte("coin")
    //   l, err := NewLeafNodeFromNibbleBytes(nibbles, value)
    //   require.NoError(t, err)
    //   require.Equal(t, "c37ec985b7a88c2c62beb268750efe657c36a585beb435eb9f43b839846682ce", fmt.Sprintf("%x", l.Hash()))
    // }
    #[test]
    fn test_leaf_node_2() {
        let nibbles = &[5, 0, 6];
        let value = b"coin";
        let l = LeafNode::new_from_nibble_bytes(nibbles, value).unwrap();
        assert_eq!(
            hex::encode(l.hash()),
            "c37ec985b7a88c2c62beb268750efe657c36a585beb435eb9f43b839846682ce"
        );
    }
}



/*




// extension.go

// type ExtensionNode struct {
//   Path []Nibble
//   Next Node
// }
pub struct ExtensionNode {
    path: Vec<Nibble>,
    next: Box<dyn Node>,
}

impl ExtensionNode {
    // func NewExtensionNode(nibbles []Nibble, next Node) *ExtensionNode {
    //   return &ExtensionNode{
    //     Path: nibbles,
    //     Next: next,
    //   }
    // }
    fn new(nibbles: Vec<Nibble>, next: Box<dyn Node>) -> ExtensionNode {
        ExtensionNode {
            path: nibbles,
            next: next,
        }
    }

    // func (e ExtensionNode) Hash() []byte {
    //   return crypto.Keccak256(e.Serialize())
    // }
    fn hash(&self) -> Vec<u8> {
        let data = self.serialize();
        let mut keccak = Keccak::new_keccak256();
        let mut hash = [0u8; 32];
        keccak.update(&data);
        keccak.finalize(&mut hash);
        hash.to_vec()
    }

    // func (e ExtensionNode) Raw() []interface{} {
    //   hashes := make([]interface{}, 2)
    //   hashes[0] = ToBytes(ToPrefixed(e.Path, false))
    //   if len(Serialize(e.Next)) >= 32 {
    //     hashes[1] = e.Next.Hash()
    //   } else {
    //     hashes[1] = e.Next.Raw()
    //   }
    //   return hashes
    // }
    fn raw(&self) -> Vec<Box<dyn Node>> {
        let mut hashes: Vec<Box<dyn Node>> = Vec::with_capacity(2);
        hashes.push(Box::new(Nibble::to_bytes(Nibble::to_prefixed(self.path.clone(), false))));
        if self.next.serialize().len() >= 32 {
            hashes.push(Box::new(self.next.hash()));
        } else {
            hashes.push(Box::new(self.next.raw()));
        }
        hashes
    }

    // func (e ExtensionNode) Serialize() []byte {
    //   return Serialize(e)
    // }
    fn serialize(&self) -> Box<dyn Node> {
        serialize(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nibble::Nibble;
    use crate::branch_node::BranchNode;
    use crate::extension_node::ExtensionNode;
    use crate::leaf_node::LeafNode;
    use hex::FromHex;

    // func TestExtensionNode(t *testing.T) {
    //   nibbles, value := []byte{5, 0, 6}, []byte("coin")
    //   leaf, err := NewLeafNodeFromNibbleBytes(nibbles, value)
    //   require.NoError(t, err)
    // 
    //   b := NewBranchNode()
    //   b.SetBranch(0, leaf)
    //   b.SetValue([]byte("verb")) // set the value for verb
    // 
    //   ns, err := FromNibbleBytes([]byte{0, 1, 0, 2, 0, 3, 0, 4})
    //   require.NoError(t, err)
    //   e := NewExtensionNode(ns, b)
    //   require.Equal(t, "e4850001020304ddc882350684636f696e8080808080808080808080808080808476657262", fmt.Sprintf("%x", e.Serialize()))
    //   require.Equal(t, "64d67c5318a714d08de6958c0e63a05522642f3f1087c6fd68a97837f203d359", fmt.Sprintf("%x", e.Hash()))
    // }
    #[test]
    fn test_extension_node() {
        let nibbles: &[u8] = &[5, 0, 6];
        let value: &[u8] = b"coin";

        let leaf = LeafNode::new_from_nibble_bytes(nibbles, value).unwrap();

        let b = BranchNode::new();
        b.set_branch(Nibble::from_byte(0), Box::new(leaf));
        b.set_value(b"verb".to_vec());

        let ns = Nibble::from_nibble_bytes(&[0, 1, 0, 2, 0, 3, 0, 4]).unwrap();
        let e = ExtensionNode::new(ns, Box::new(b));

        let expected_serialize: [u8; 20] = FromHex::from_hex("e4850001020304ddc882350684636f696e8080808080808080808080808080808476657262").unwrap();
        assert_eq!(e.serialize(), expected_serialize);

        let expected_hash: [u8; 32] = FromHex::from_hex("64d67c5318a714d08de6958c0e63a05522642f3f1087c6fd68a97837f203d359").unwrap();
        assert_eq!(e.hash(), expected_hash);
    }
}


// branch.go

use std::cell::RefCell;

// type BranchNode struct {
//   Branches [16]Node
//   Value    []byte
// }
pub struct BranchNode {
    branches: RefCell<[Option<Box<dyn Node>>; 16]>,
    value: RefCell<Option<Vec<u8>>>,
}

impl Node for BranchNode {
    fn hash(&self) -> Vec<u8> {
        self.hash()
    }

    fn raw(&self) -> Vec<Box<dyn Node>> {
        self.raw()
    }

    fn serialize(&self) -> Box<dyn Node> {
        self.serialize()
    }
}

impl BranchNode {
    // func NewBranchNode() *BranchNode {
    //   return &BranchNode{
    //     Branches: [16]Node{},
    //   }
    // }
    fn new() -> BranchNode {
        BranchNode {
            branches: RefCell::new([None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None]),
            value: RefCell::new(None),
        }
    }

    // func (b BranchNode) Hash() []byte {
    //   return crypto.Keccak256(b.Serialize())
    // }
    fn hash(&self) -> Vec<u8> {
        let data = self.serialize();
        let mut keccak = Keccak::new_keccak256();
        let mut hash = [0u8; 32];
        keccak.update(&data);
        keccak.finalize(&mut hash);
        hash.to_vec()
    }

    // func (b *BranchNode) SetBranch(nibble Nibble, node Node) {
    //   b.Branches[int(nibble)] = node
    // }
    fn set_branch(&self, nibble: Nibble, node: Box<dyn Node>) {
        self.branches.borrow_mut()[nibble.to_usize()] = Some(node);
    }

    // func (b *BranchNode) RemoveBranch(nibble Nibble) {
    //   b.Branches[int(nibble)] = nil
    // }
    fn remove_branch(&self, nibble: Nibble) {
        self.branches.borrow_mut()[nibble.to_usize()] = None;
    }

    // func (b *BranchNode) SetValue(value []byte) {
    //   b.Value = value
    // }
    fn set_value(&self, value: Vec<u8>) {
        *self.value.borrow_mut() = Some(value);
    }

    // func (b *BranchNode) RemoveValue() {
    //   b.Value = nil
    // }
    fn remove_value(&self) {
        *self.value.borrow_mut() = None;
    }

    // func (b BranchNode) Raw() []interface{} {
    //   hashes := make([]interface{}, 17)
    //   for i := 0; i < 16; i++ {
    //     if b.Branches[i] == nil {
    //       hashes[i] = EmptyNodeRaw
    //     } else {
    //       node := b.Branches[i]
    //       if len(Serialize(node)) >= 32 {
    //         hashes[i] = node.Hash()
    //       } else {
    //         // if node can be serialized to less than 32 bits, then
    //         // use Serialized directly.
    //         // it has to be ">=", rather than ">",
    //         // so that when deserialized, the content can be distinguished
    //         // by length
    //         hashes[i] = node.Raw()
    //       }
    //     }
    //   }
    // 
    //   hashes[16] = b.Value
    //   return hashes
    // }
    fn raw(&self) -> Vec<Vec<u8>> {
        let mut hashes: Vec<Vec<u8>> = Vec::with_capacity(17);
        for i in 0..16 {
            match &self.branches.borrow()[i] {
                None => hashes[i] = EMPTY_NODE_RAW,
                Some(node) => {
                    if node.serialize().len() >= 32 {
                        hashes[i] = node.hash();
                    } else {
                        hashes[i] = node.raw();
                    }
                }
            }
        }

        hashes[16] = LeafNode::new_from_bytes(&[], &self.value.borrow().as_ref().unwrap());
        hashes
    }

    // func (b BranchNode) Serialize() []byte {
    //   return Serialize(b)
    // }
    fn serialize(&self) -> Vec<u8> {
        serialize(self)
    }

    // func (b BranchNode) HasValue() bool {
    //   return b.Value != nil
    // }
    fn has_value(&self) -> bool {
        self.value.borrow().is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trie::{BranchNode, LeafNode, Nibble, Node, Trie};

    // func TestBranch(t *testing.T) {
    //   nibbles, value := []byte{5, 0, 6}, []byte("coin")
    //   leaf, err := NewLeafNodeFromNibbleBytes(nibbles, value)
    //   require.NoError(t, err)
    // 
    //   b := NewBranchNode()
    //   b.SetBranch(0, leaf)
    //   b.SetValue([]byte("verb")) // set the value for verb
    // 
    //   require.Equal(t, "ddc882350684636f696e8080808080808080808080808080808476657262",
    //     fmt.Sprintf("%x", b.Serialize()))
    //   require.Equal(t, "d757709f08f7a81da64a969200e59ff7e6cd6b06674c3f668ce151e84298aa79",
    //     fmt.Sprintf("%x", b.Hash()))
    // 
    // }
    #[test]
    fn test_branch() {
        let nibbles = vec![5, 0, 6];
        let value = b"coin";
        let leaf = LeafNode::from_nibble_bytes(&nibbles, value).unwrap();

        let mut b = BranchNode::new();
        b.set_branch(Nibble(0), Node::Leaf(leaf));
        b.set_value(b"verb");

        assert_eq!(
            "ddc882350684636f696e8080808080808080808080808080808476657262",
            hex::encode(b.serialize())
        );
        assert_eq!(
            "d757709f08f7a81da64a969200e59ff7e6cd6b06674c3f668ce151e84298aa79",
            hex::encode(b.hash())
        );
    }
}

// trie.go

// #[derive(Clone)]
// pub enum Node {
//     Empty,
//     Leaf(LeafNode),
//     Branch(BranchNode),
//     Extension(ExtensionNode),
// }

pub struct Trie {
//     root: Node,
    root: Box<dyn Node>,
}

impl Trie {
    // func NewTrie() *Trie {
    //   return &Trie{}
    // }
    pub fn new() -> Trie {
        Trie { root: Node::Empty }
    }

    // func (t *Trie) Hash() []byte {
    //   if IsEmptyNode(t.root) {
    //     return EmptyNodeHash
    //   }
    //   return t.root.Hash()
    // }
    pub fn hash(&self) -> Box<dyn Any> {
        match is_empty_node(&self.root) {
            true => EMPTY_NODE_HASH.to_vec(),
            false => self.root.as_ref().unwrap().hash(),
        }
    }

    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        // node := t.root
        // nibbles := FromBytes(key)
        let mut node = &self.root;
        let mut nibbles = Nibble::from_bytes(key);

        loop {
            match node {
                // if IsEmptyNode(node) {
                //   return nil, false
                // }
                <dyn Node>::Empty => return None,

                // if leaf, ok := node.(*LeafNode); ok {
                //   matched := PrefixMatchedLen(leaf.Path, nibbles)
                //   if matched != len(leaf.Path) || matched != len(nibbles) {
                //     return nil, false
                //   }
                //   return leaf.Value, true
                // }
                Node::Leaf(leaf) => {
                    let matched = Nibble::prefix_matched_len(&leaf.path, &nibbles);
                    if matched != leaf.path.len() || matched != nibbles.len() {
                        return None;
                    }
                    return Some(leaf.value.clone());
                }

                // if branch, ok := node.(*BranchNode); ok {
                //   if len(nibbles) == 0 {
                //     return branch.Value, branch.HasValue()
                //   }
                // 
                //   b, remaining := nibbles[0], nibbles[1:]
                //   nibbles = remaining
                //   node = branch.Branches[b]
                //   continue
                // }
                Node::Branch(branch) => {
                    if nibbles.is_empty() {
                        return branch.value.clone();
                    }
                    let (b, remaining) = nibbles.split_at(1);
                    nibbles = remaining.to_vec();
                    node = &branch.branches[b[0] as usize];
                }

                // if ext, ok := node.(*ExtensionNode); ok {
                //   matched := PrefixMatchedLen(ext.Path, nibbles)
                //   // E 01020304
                //   //   010203
                //   if matched < len(ext.Path) {
                //     return nil, false
                //   }
                // 
                //   nibbles = nibbles[matched:]
                //   node = ext.Next
                //   continue
                // }
                Node::Extension(ext) => {
                    let matched = Nibble::prefix_matched_len(&ext.path, &nibbles);
                    if matched < ext.path.len() {
                        return None;
                    }
                    nibbles = nibbles[matched..].to_vec();
                    node = &ext.next;
                }
            }
        }
    }

    // Put adds a key value pair to the trie
    // In general, the rule is:
    // - When stopped at an EmptyNode, replace it with a new LeafNode with the remaining path.
    // - When stopped at a LeafNode, convert it to an ExtensionNode and add a new branch and a new LeafNode.
    // - When stopped at an ExtensionNode, convert it to another ExtensionNode with shorter path and create a new BranchNode points to the ExtensionNode.
    pub fn put(&mut self, key: &[u8], value: &[u8]) -> Result<(), &'static str> {
        // // need to use pointer, so that I can update root in place without
        // // keeping trace of the parent node
        // node := &t.root
        // nibbles := FromBytes(key)
        let mut node = &mut self.root;
        let mut nibbles = Nibble::from_bytes(key);

        loop {
            // if IsEmptyNode(*node) {
            //   leaf := NewLeafNodeFromNibbles(nibbles, value)
            //   *node = leaf
            //   return
            // }
            if is_empty_node(&node) {
                *node = Box::new(LeafNode::new_leaf_node_from_nibbles(nibbles.clone(), value.to_vec()));
                return Ok(());
            }

            if let Node::Leaf(ref mut leaf) = node {
                // matched := PrefixMatchedLen(leaf.Path, nibbles)
                let matched = Nibble::prefix_matched_len(&leaf.path, &nibbles);

                // // if all matched, update value even if the value are equal
                // if matched == len(nibbles) && matched == len(leaf.Path) {
                //   newLeaf := NewLeafNodeFromNibbles(leaf.Path, value)
                //   *node = newLeaf
                //   return
                // }
                if matched == nibbles.len() && matched == leaf.path.len() {
                    *node = Box::new(LeafNode::new_leaf_node_from_nibbles(leaf.path.clone(), value.to_vec()));
                    return Ok(());
                }

                // branch := NewBranchNode()
                let mut branch = BranchNode::new();

                // // if matched some nibbles, check if matches either all remaining nibbles
                // // or all leaf nibbles
                // if matched == len(leaf.Path) {
                //   branch.SetValue(leaf.Value)
                // }
                if matched == leaf.path.len() {
                    branch.set_value(leaf.value.clone());
                }

                // if matched == len(nibbles) {
                //   branch.SetValue(value)
                // }
                if matched == nibbles.len() {
                    branch.set_value(value.to_vec());
                }

                // // if there is matched nibbles, an extension node will be created
                // if matched > 0 {
                //   // create an extension node for the shared nibbles
                //   ext := NewExtensionNode(leaf.Path[:matched], branch)
                //   *node = ext
                // } else {
                //   // when there no matched nibble, there is no need to keep the extension node
                //   *node = branch
                // }
                if matched > 0 {
                    *node = Box::new(ExtensionNode::new(leaf.path[..matched].to_vec(), Box::new(branch)));
                } else {
                    *node = Box::new(branch);
                }

                // if matched < len(leaf.Path) {
                //   // have dismatched
                //   // L 01020304 hello
                //   // + 010203   world
                // 
                //   // 01020304, 0, 4
                //   branchNibble, leafNibbles := leaf.Path[matched], leaf.Path[matched+1:]
                //   newLeaf := NewLeafNodeFromNibbles(leafNibbles, leaf.Value) // not :matched+1
                //   branch.SetBranch(branchNibble, newLeaf)
                // }
                if matched < leaf.path.len() {
                    let (branch_nibble, leaf_nibbles) = leaf.path.split_at(matched + 1);
                    branch.set_branch(branch_nibble[0], Box::new(LeafNode::new_leaf_node_from_nibbles(leaf_nibbles.to_vec(), leaf.value.clone())));
                }

                // if matched < len(nibbles) {
                //   // L 01020304 hello
                //   // + 010203040 world
                // 
                //   // L 01020304 hello
                //   // + 010203040506 world
                //   branchNibble, leafNibbles := nibbles[matched], nibbles[matched+1:]
                //   newLeaf := NewLeafNodeFromNibbles(leafNibbles, value)
                //   branch.SetBranch(branchNibble, newLeaf)
                // }
                if matched < nibbles.len() {
                    let (branch_nibble, leaf_nibbles) = nibbles.split_at(matched + 1);
                    branch.set_branch(branch_nibble[0], Box::new(LeafNode::new_leaf_node_from_nibbles(leaf_nibbles.to_vec(), value.to_vec())));
                }

                return Ok(());
            }

            // if branch, ok := (*node).(*BranchNode); ok {
            //   if len(nibbles) == 0 {
            //     branch.SetValue(value)
            //     return
            //   }
            // 
            //   b, remaining := nibbles[0], nibbles[1:]
            //   nibbles = remaining
            //   node = &branch.Branches[b]
            //   continue
            // }
            if let Node::Branch(ref mut branch) = node {
                if nibbles.is_empty() {
                    branch.set_value(value.to_vec());
                    return Ok(());
                }

                let (b, remaining) = nibbles.split_at(1);
                nibbles = remaining.to_vec();
                node = &mut branch.branches[b[0] as usize];
                continue;
            }

            // E 01020304
            // B 0 hello
            // L 506 world
            // + 010203 good
            // if ext, ok := (*node).(*ExtensionNode); ok {
            if let Node::Extension(ref mut ext) = node {
                // matched := PrefixMatchedLen(ext.Path, nibbles)
                // if matched < len(ext.Path) {
                let matched = Nibble::prefix_matched_len(&ext.path, &nibbles);
                if matched < ext.path.len() {
                    // extNibbles, branchNibble, extRemainingnibbles := ext.Path[:matched], ext.Path[matched], ext.Path[matched+1:]
                    // branch := NewBranchNode()
                    let (ext_nibbles, remaining) = ext.path.split_at(matched);
                    let (branch_nibble, ext_remaining_nibbles) = remaining.split_at(1);
                    let mut branch = BranchNode::new();

                    // if len(extRemainingnibbles) == 0 {
                    //   // E 0102030
                    //   // + 010203 good
                    //   branch.SetBranch(branchNibble, ext.Next)
                    // } else {
                    //   // E 01020304
                    //   // + 010203 good
                    //   newExt := NewExtensionNode(extRemainingnibbles, ext.Next)
                    //   branch.SetBranch(branchNibble, newExt)
                    // }
                    if ext_remaining_nibbles.is_empty() {
                        branch.set_branch(branch_nibble[0], ext.next.clone());
                    } else {
                        branch.set_branch(branch_nibble[0], Box::new(ExtensionNode::new(ext_remaining_nibbles.to_vec(), ext.next.clone())));
                    }

                    // if matched < len(nibbles) {
                    //   nodeBranchNibble, nodeLeafNibbles := nibbles[matched], nibbles[matched+1:]
                    //   remainingLeaf := NewLeafNodeFromNibbles(nodeLeafNibbles, value)
                    //   branch.SetBranch(nodeBranchNibble, remainingLeaf)
                    // } else if matched == len(nibbles) {
                    //   branch.SetValue(value)
                    // } else {
                    //   panic(fmt.Sprintf("too many matched (%v > %v)", matched, len(nibbles)))
                    // }
                    if matched < nibbles.len() {
                        let (node_branch_nibble, node_leaf_nibbles) = nibbles.split_at(matched + 1);
                        let remaining_leaf = LeafNode::new_leaf_node_from_nibbles(node_leaf_nibbles.to_vec(), value.to_vec());
                        branch.set_branch(node_branch_nibble[0], Box::new(remaining_leaf));
                    } else if matched == nibbles.len() {
                        branch.set_value(value.to_vec());
                    } else {
                        return Err("Too many matches");
                    }

                    // // if there is no shared extension nibbles any more, then we don't need the extension node
                    // // any more
                    // // E 01020304
                    // // + 1234 good
                    // if len(extNibbles) == 0 {
                    //   *node = branch
                    // } else {
                    //   // otherwise create a new extension node
                    //   *node = NewExtensionNode(extNibbles, branch)
                    // }
                    if ext_nibbles.is_empty() {
                        *node = Box::new(branch);
                    } else {
                        *node = Box::new(ExtensionNode::new(ext_nibbles.to_vec(), Box::new(branch)));
                    }
                    return Ok(());
                }

                // nibbles = nibbles[matched:]
                // node = &ext.Next
                // continue
                let (_, remaining) = nibbles.split_at(matched);
                nibbles = remaining.to_vec();
                node = &mut ext.next;
                continue;
            }
            
            // panic("unknown type")
            return Err("Unknown type");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trie::Trie;
    use crate::trie::{Trie, Nibble};

    // func hexEqual(t *testing.T, hex string, bytes []byte) {
    //   require.Equal(t, hex, fmt.Sprintf("%x", bytes))
    // }
    fn hex_equal(expected_hex: &str, actual: &[u8]) {
        let expected = hex::decode(expected_hex).unwrap();
        assert_eq!(expected, actual);
    }

    // // check basic key-value mapping
    // func TestGetPut(t *testing.T) {
    //   t.Run("should get nothing if key does not exist", func(t *testing.T) {
    //     trie := NewTrie()
    //     _, found := trie.Get([]byte("notexist"))
    //     require.Equal(t, false, found)
    //   })
    // 
    //   t.Run("should get value if key exist", func(t *testing.T) {
    //     trie := NewTrie()
    //     trie.Put([]byte{1, 2, 3, 4}, []byte("hello"))
    //     val, found := trie.Get([]byte{1, 2, 3, 4})
    //     require.Equal(t, true, found)
    //     require.Equal(t, val, []byte("hello"))
    //   })
    // 
    //   t.Run("should get updated value", func(t *testing.T) {
    //     trie := NewTrie()
    //     trie.Put([]byte{1, 2, 3, 4}, []byte("hello"))
    //     trie.Put([]byte{1, 2, 3, 4}, []byte("world"))
    //     val, found := trie.Get([]byte{1, 2, 3, 4})
    //     require.Equal(t, true, found)
    //     require.Equal(t, val, []byte("world"))
    //   })
    // }
    #[test]
    fn test_get_put() {
        let mut trie = Trie::new();

        // should get nothing if key does not exist
        assert_eq!(trie.get(&[1, 2, 3, 4]), None);

        // should get value if key exists
        trie.put(&[1, 2, 3, 4], b"hello");
        assert_eq!(trie.get(&[1, 2, 3, 4]), Some(b"hello".to_vec()));

        // should get updated value
        trie.put(&[1, 2, 3, 4], b"world");
        assert_eq!(trie.get(&[1, 2, 3, 4]), Some(b"world".to_vec()));
    }

    // // verify data integrity
    // func TestDataIntegrity(t *testing.T) {
    //   t.Run("should get a different hash if a new key-value pair was added or updated", func(t *testing.T) {
    //     trie := NewTrie()
    //     hash0 := trie.Hash()
    // 
    //     trie.Put([]byte{1, 2, 3, 4}, []byte("hello"))
    //     hash1 := trie.Hash()
    // 
    //     trie.Put([]byte{1, 2}, []byte("world"))
    //     hash2 := trie.Hash()
    // 
    //     trie.Put([]byte{1, 2}, []byte("trie"))
    //     hash3 := trie.Hash()
    // 
    //     require.NotEqual(t, hash0, hash1)
    //     require.NotEqual(t, hash1, hash2)
    //     require.NotEqual(t, hash2, hash3)
    //   })
    // 
    //   t.Run("should get the same hash if two tries have the identicial key-value pairs", func(t *testing.T) {
    //     trie1 := NewTrie()
    //     trie1.Put([]byte{1, 2, 3, 4}, []byte("hello"))
    //     trie1.Put([]byte{1, 2}, []byte("world"))
    // 
    //     trie2 := NewTrie()
    //     trie2.Put([]byte{1, 2, 3, 4}, []byte("hello"))
    //     trie2.Put([]byte{1, 2}, []byte("world"))
    // 
    //     require.Equal(t, trie1.Hash(), trie2.Hash())
    //   })
    // }
    #[test]
    fn test_data_integrity() {
        let mut trie = Trie::new();

        // should get a different hash if a new key-value pair was added or updated
        let hash0 = trie.hash();
        trie.put(&[1, 2, 3, 4], b"hello");
        let hash1 = trie.hash();
        trie.put(&[1, 2], b"world");
        let hash2 = trie.hash();
        trie.put(&[1, 2], b"trie");
        let hash3 = trie.hash();

        assert_ne!(hash0, hash1);
        assert_ne!(hash1, hash2);
        assert_ne!(hash2, hash3);

        // should get the same hash if two tries have identical key-value pairs
        let mut trie1 = Trie::new();
        trie1.put(&[1, 2, 3, 4], b"hello");
        trie1.put(&[1, 2], b"world");

        let mut trie2 = Trie::new();
        trie2.put(&[1, 2, 3, 4], b"hello");
        trie2.put(&[1, 2], b"world");

        assert_eq!(trie1.hash(), trie2.hash());
    }

    // func TestPut2Pairs(t *testing.T) {
    //   trie := NewTrie()
    //   trie.Put([]byte{1, 2, 3, 4}, []byte("verb"))
    //   trie.Put([]byte{1, 2, 3, 4, 5, 6}, []byte("coin"))
    // 
    //   verb, ok := trie.Get([]byte{1, 2, 3, 4})
    //   require.True(t, ok)
    //   require.Equal(t, []byte("verb"), verb)
    // 
    //   coin, ok := trie.Get([]byte{1, 2, 3, 4, 5, 6})
    //   require.True(t, ok)
    //   require.Equal(t, []byte("coin"), coin)
    // 
    //   fmt.Printf("%T\n", trie.root)
    //   ext, ok := trie.root.(*ExtensionNode)
    //   require.True(t, ok)
    //   branch, ok := ext.Next.(*BranchNode)
    //   require.True(t, ok)
    //   leaf, ok := branch.Branches[0].(*LeafNode)
    //   require.True(t, ok)
    // 
    //   hexEqual(t, "c37ec985b7a88c2c62beb268750efe657c36a585beb435eb9f43b839846682ce", leaf.Hash())
    //   hexEqual(t, "ddc882350684636f696e8080808080808080808080808080808476657262", branch.Serialize())
    //   hexEqual(t, "d757709f08f7a81da64a969200e59ff7e6cd6b06674c3f668ce151e84298aa79", branch.Hash())
    //   hexEqual(t, "64d67c5318a714d08de6958c0e63a05522642f3f1087c6fd68a97837f203d359", ext.Hash())
    // }
    #[test]
    fn test_put_2_pairs() {
        let mut trie = Trie::new();
        trie.put(&[1, 2, 3, 4], b"verb");
        trie.put(&[1, 2, 3, 4, 5, 6], b"coin");

        assert_eq!(trie.get(&[1, 2, 3, 4]), Some(b"verb".to_vec()));
        assert_eq!(trie.get(&[1, 2, 3, 4, 5, 6]), Some(b"coin".to_vec()));

        assert!(matches!(trie.root, Node::Extension(ext) => {
            matches!(ext.next, Node::Branch(branch) => {
                matches!(branch.branches[0], Node::Leaf(leaf) => {
                    hex_equal("c37ec985b7a88c2c62beb268750efe657c36a585beb435eb9f43b839846682ce", leaf.hash());
                    hex_equal("ddc882350684636f696e8080808080808080808080808080808476657262", branch.serialize());
                    hex_equal("d757709f08f7a81da64a969200e59ff7e6cd6b06674c3f668ce151e84298aa79", branch.hash());
                    hex_equal("64d67c5318a714d08de6958c0e63a05522642f3f1087c6fd68a97837f203d359", ext.hash());
                    true
                })
            })
        }));
    }

    // func TestPut(t *testing.T) {
    //   trie := NewTrie()
    //   require.Equal(t, EmptyNodeHash, trie.Hash())
    //   trie.Put([]byte{1, 2, 3, 4}, []byte("hello"))
    //   ns := NewLeafNodeFromBytes([]byte{1, 2, 3, 4}, []byte("hello"))
    //   require.Equal(t, ns.Hash(), trie.Hash())
    // }
    #[test]
    fn test_put() {
        let mut trie = Trie::new();
        assert_eq!(EmptyNodeHash, trie.hash());

        trie.put(&[1, 2, 3, 4], b"hello");
        let ns = LeafNode::from_bytes(&[1, 2, 3, 4], b"hello");
        assert_eq!(ns.hash(), trie.hash());
    }

    // func TestPutLeafShorter(t *testing.T) {
    //   trie := NewTrie()
    //   trie.Put([]byte{1, 2, 3, 4}, []byte("hello"))
    //   trie.Put([]byte{1, 2, 3}, []byte("world"))
    // 
    //   leaf := NewLeafNodeFromNibbles([]Nibble{4}, []byte("hello"))
    // 
    //   branch := NewBranchNode()
    //   branch.SetBranch(Nibble(0), leaf)
    //   branch.SetValue([]byte("world"))
    // 
    //   ext := NewExtensionNode([]Nibble{0, 1, 0, 2, 0, 3}, branch)
    // 
    //   require.Equal(t, ext.Hash(), trie.Hash())
    // }
    #[test]
    fn test_put_leaf_shorter() {
        let mut trie = Trie::new();
        trie.put(&[1, 2, 3, 4], b"hello");
        trie.put(&[1, 2, 3], b"world");

        let leaf = LeafNode::from_nibbles(&[4], b"hello");

        let mut branch = BranchNode::new();
        branch.set_branch(Nibble(0), Node::Leaf(leaf));
        branch.set_value(b"world");

        let ext = ExtensionNode::new(&[0, 1, 0, 2, 0, 3], Node::Branch(branch));

        assert_eq!(ext.hash(), trie.hash());
    }

    // func TestPutLeafAllMatched(t *testing.T) {
    //   trie := NewTrie()
    //   trie.Put([]byte{1, 2, 3, 4}, []byte("hello"))
    //   trie.Put([]byte{1, 2, 3, 4}, []byte("world"))
    // 
    //   ns := NewLeafNodeFromBytes([]byte{1, 2, 3, 4}, []byte("world"))
    //   require.Equal(t, ns.Hash(), trie.Hash())
    // }
    #[test]
    fn test_put_leaf_all_matched() {
        let mut trie = Trie::new();
        trie.put(&[1, 2, 3, 4], b"hello");
        trie.put(&[1, 2, 3, 4], b"world");

        let ns = LeafNode::from_bytes(&[1, 2, 3, 4], b"world");
        assert_eq!(ns.hash(), trie.hash());
    }

    // func TestPutLeafMore(t *testing.T) {
    //   trie := NewTrie()
    //   trie.Put([]byte{1, 2, 3, 4}, []byte("hello"))
    //   trie.Put([]byte{1, 2, 3, 4, 5, 6}, []byte("world"))
    // 
    //   leaf := NewLeafNodeFromNibbles([]Nibble{5, 0, 6}, []byte("world"))
    // 
    //   branch := NewBranchNode()
    //   branch.SetValue([]byte("hello"))
    //   branch.SetBranch(Nibble(0), leaf)
    // 
    //   ext := NewExtensionNode([]Nibble{0, 1, 0, 2, 0, 3, 0, 4}, branch)
    // 
    //   require.Equal(t, ext.Hash(), trie.Hash())
    // }
    #[test]
    fn test_put_leaf_more() {
        let mut trie = Trie::new();
        trie.put(&[1, 2, 3, 4], b"hello");
        trie.put(&[1, 2, 3, 4, 5, 6], b"world");

        let leaf = LeafNode::from_nibbles(&[5, 0, 6], b"world");

        let mut branch = BranchNode::new();
        branch.set_value(b"hello");
        branch.set_branch(Nibble(0), Node::Leaf(leaf));

        let ext = ExtensionNode::new(&[0, 1, 0, 2, 0, 3, 0, 4], Node::Branch(branch));

        assert_eq!(ext.hash(), trie.hash());
    }

    // func TestPutOrder(t *testing.T) {
    //   trie1, trie2 := NewTrie(), NewTrie()
    // 
    //   trie1.Put([]byte{1, 2, 3, 4, 5, 6}, []byte("world"))
    //   trie1.Put([]byte{1, 2, 3, 4}, []byte("hello"))
    // 
    //   trie2.Put([]byte{1, 2, 3, 4}, []byte("hello"))
    //   trie2.Put([]byte{1, 2, 3, 4, 5, 6}, []byte("world"))
    // 
    //   require.Equal(t, trie1.Hash(), trie2.Hash())
    // }
    #[test]
    fn test_put_order() {
        let mut trie1 = Trie::new();
        let mut trie2 = Trie::new();

        trie1.put(&[1, 2, 3, 4, 5, 6], b"world");
        trie1.put(&[1, 2, 3, 4], b"hello");

        trie2.put(&[1, 2, 3, 4], b"hello");
        trie2.put(&[1, 2, 3, 4, 5, 6], b"world");

        assert_eq!(trie1.hash(), trie2.hash());
    }

    // Before put:
    //
    //               ┌───────────────────────────┐
    //               │  Extension Node           │
    //               │  Path: [0, 1, 0, 2, 0, 3] │
    //               └────────────┬──────────────┘
    //                            │
    //    ┌───────────────────────┴──────────────────┐
    //    │                   Branch Node            │
    //    │   [0]         ...          [5]           │
    //    └────┼────────────────────────┼────────────┘
    //         │                        │
    //         │                        │
    //         │                        │
    //         │                        │
    //   ┌───────┴──────────┐   ┌─────────┴─────────┐
    //   │  Leaf Node       │   │  Leaf Node        │
    //   │  Path: [4]       │   │  Path: [0]        │
    //   │  Value: "hello1" │   │  Value: "hello2"  │
    //   └──────────────────┘   └───────────────────┘
    //
    // After put([]byte{[1, 2, 3]}, "world"):
    //               ┌───────────────────────────┐
    //               │  Extension Node           │
    //               │  Path: [0, 1, 0, 2, 0, 3] │
    //               └────────────┬──────────────┘
    //                            │
    //    ┌───────────────────────┴────────────────────────┐
    //    │                   Branch Node                  │
    //    │   [0]         ...          [5]  value: "world" │
    //    └────┼────────────────────────┼──────────────────┘
    //         │                        │
    //         │                        │
    //         │                        │
    //         │                        │
    //   ┌───────┴──────────┐   ┌─────────┴─────────┐
    //   │  Leaf Node       │   │  Leaf Node        │
    //   │  Path: [4]       │   │  Path: [0]        │
    //   │  Value: "hello1" │   │  Value: "hello2"  │
    //   └──────────────────┘   └───────────────────┘
    // func TestPutExtensionShorterAllMatched(t *testing.T) {
    //   trie := NewTrie()
    //   trie.Put([]byte{1, 2, 3, 4}, []byte("hello1"))
    //   trie.Put([]byte{1, 2, 3, 5}, []byte("hello2"))
    //   trie.Put([]byte{1, 2, 3}, []byte("world"))
    // 
    //   leaf1 := NewLeafNodeFromNibbles([]Nibble{}, []byte("hello1"))
    //   leaf2 := NewLeafNodeFromNibbles([]Nibble{}, []byte("hello2"))
    // 
    //   branch1 := NewBranchNode()
    //   branch1.SetBranch(Nibble(4), leaf1)
    //   branch1.SetBranch(Nibble(5), leaf2)
    // 
    //   branch2 := NewBranchNode()
    //   branch2.SetValue([]byte("world"))
    //   branch2.SetBranch(Nibble(0), branch1)
    // 
    //   ext := NewExtensionNode([]Nibble{0, 1, 0, 2, 0, 3}, branch2)
    // 
    //   require.Equal(t, ext.Hash(), trie.Hash())
    // }
    #[test]
    fn test_put_extension_shorter_all_matched() {
        let mut trie = Trie::new();
        trie.put(&[1, 2, 3, 4], b"hello1");
        trie.put(&[1, 2, 3, 5], b"hello2");
        trie.put(&[1, 2, 3], b"world");

        let leaf1 = LeafNode::from_nibbles(&[], b"hello1");
        let leaf2 = LeafNode::from_nibbles(&[], b"hello2");

        let mut branch1 = BranchNode::new();
        branch1.set_branch(Nibble(4), Node::Leaf(leaf1));
        branch1.set_branch(Nibble(5), Node::Leaf(leaf2));

        let mut branch2 = BranchNode::new();
        branch2.set_value(b"world");
        branch2.set_branch(Nibble(0), Node::Branch(branch1));

        let ext = ExtensionNode::new(&[0, 1, 0, 2, 0, 3], Node::Branch(branch2));

        assert_eq!(ext.hash(), trie.hash());
    }

    // func TestPutExtensionShorterPartialMatched(t *testing.T) {
    //   trie := NewTrie()
    //   trie.Put([]byte{1, 2, 3, 4}, []byte("hello1"))
    //   trie.Put([]byte{1, 2, 3, 5}, []byte("hello2"))
    //   trie.Put([]byte{1, 2, 5}, []byte("world"))
    // 
    //   leaf1 := NewLeafNodeFromNibbles([]Nibble{}, []byte("hello1"))
    //   leaf2 := NewLeafNodeFromNibbles([]Nibble{}, []byte("hello2"))
    // 
    //   branch1 := NewBranchNode()
    //   branch1.SetBranch(Nibble(4), leaf1)
    //   branch1.SetBranch(Nibble(5), leaf2)
    // 
    //   ext1 := NewExtensionNode([]Nibble{0}, branch1)
    // 
    //   branch2 := NewBranchNode()
    //   branch2.SetBranch(Nibble(3), ext1)
    //   leaf3 := NewLeafNodeFromNibbles([]Nibble{}, []byte("world"))
    //   branch2.SetBranch(Nibble(5), leaf3)
    // 
    //   ext2 := NewExtensionNode([]Nibble{0, 1, 0, 2, 0}, branch2)
    // 
    //   require.Equal(t, ext2.Hash(), trie.Hash())
    // }
    #[test]
    fn test_put_extension_shorter_partial_matched() {
        let mut trie = Trie::new();
        trie.put(&[1, 2, 3, 4], b"hello1");
        trie.put(&[1, 2, 3, 5], b"hello2");
        trie.put(&[1, 2, 5], b"world");

        let leaf1 = LeafNode::from_nibbles(&[], b"hello1");
        let leaf2 = LeafNode::from_nibbles(&[], b"hello2");

        let mut branch1 = BranchNode::new();
        branch1.set_branch(Nibble(4), Node::Leaf(leaf1));
        branch1.set_branch(Nibble(5), Node::Leaf(leaf2));

        let ext1 = ExtensionNode::new(&[0], Node::Branch(branch1));

        let mut branch2 = BranchNode::new();
        branch2.set_branch(Nibble(3), Node::Extension(ext1));
        let leaf3 = LeafNode::from_nibbles(&[], b"world");
        branch2.set_branch(Nibble(5), Node::Leaf(leaf3));

        let ext2 = ExtensionNode::new(&[0, 1, 0, 2, 0], Node::Branch(branch2));

        assert_eq!(ext2.hash(), trie.hash());
    }

    // func TestPutExtensionShorterZeroMatched(t *testing.T) {
    //   trie := NewTrie()
    //   trie.Put([]byte{1, 2, 3, 4}, []byte("hello1"))
    //   trie.Put([]byte{1, 2, 3, 5}, []byte("hello2"))
    //   trie.Put([]byte{1 << 4, 2, 5}, []byte("world"))
    // 
    //   leaf1 := NewLeafNodeFromNibbles([]Nibble{}, []byte("hello1"))
    //   leaf2 := NewLeafNodeFromNibbles([]Nibble{}, []byte("hello2"))
    // 
    //   branch1 := NewBranchNode()
    //   branch1.SetBranch(Nibble(4), leaf1)
    //   branch1.SetBranch(Nibble(5), leaf2)
    // 
    //   ext1 := NewExtensionNode([]Nibble{1, 0, 2, 0, 3, 0}, branch1)
    // 
    //   branch2 := NewBranchNode()
    //   branch2.SetBranch(Nibble(0), ext1)
    //   leaf3 := NewLeafNodeFromNibbles([]Nibble{0, 0, 2, 0, 5}, []byte("world"))
    //   branch2.SetBranch(Nibble(1), leaf3)
    // 
    //   require.Equal(t, branch2.Hash(), trie.Hash())
    // }
    #[test]
    fn test_put_extension_shorter_zero_matched() {
        let mut trie = Trie::new();
        trie.put(&[1, 2, 3, 4], b"hello1");
        trie.put(&[1, 2, 3, 5], b"hello2");
        trie.put(&[1 << 4, 2, 5], b"world");

        let leaf1 = LeafNode::from_nibbles(&[], b"hello1");
        let leaf2 = LeafNode::from_nibbles(&[], b"hello2");

        let mut branch1 = BranchNode::new();
        branch1.set_branch(Nibble(4), Node::Leaf(leaf1));
        branch1.set_branch(Nibble(5), Node::Leaf(leaf2));

        let ext1 = ExtensionNode::new(&[1, 0, 2, 0, 3, 0], Node::Branch(branch1));

        let mut branch2 = BranchNode::new();
        branch2.set_branch(Nibble(0), Node::Extension(ext1));
        let leaf3 = LeafNode::from_nibbles(&[0, 0, 2, 0, 5], b"world");
        branch2.set_branch(Nibble(1), Node::Leaf(leaf3));

        assert_eq!(branch2.hash(), trie.hash());
    }

    // func TestPutExtensionAllMatched(t *testing.T) {
    //   trie := NewTrie()
    //   trie.Put([]byte{1, 2, 3, 4}, []byte("hello1"))
    //   trie.Put([]byte{1, 2, 3, 5 << 4}, []byte("hello2"))
    //   trie.Put([]byte{1, 2, 3}, []byte("world"))
    // 
    //   leaf1 := NewLeafNodeFromNibbles([]Nibble{4}, []byte("hello1"))
    //   leaf2 := NewLeafNodeFromNibbles([]Nibble{0}, []byte("hello2"))
    // 
    //   branch := NewBranchNode()
    //   branch.SetBranch(Nibble(0), leaf1)
    //   branch.SetBranch(Nibble(5), leaf2)
    //   branch.SetValue([]byte("world"))
    // 
    //   ext := NewExtensionNode([]Nibble{0, 1, 0, 2, 0, 3}, branch)
    // 
    //   require.Equal(t, ext.Hash(), trie.Hash())
    // }
    #[test]
    fn test_put_extension_all_matched() {
        let mut trie = Trie::new();
        trie.put(&[1, 2, 3, 4], b"hello1");
        trie.put(&[1, 2, 3, 5 << 4], b"hello2");
        trie.put(&[1, 2, 3], b"world");

        let leaf1 = LeafNode::from_nibbles(&[4], b"hello1");
        let leaf2 = LeafNode::from_nibbles(&[0], b"hello2");

        let mut branch = BranchNode::new();
        branch.set_branch(Nibble(0), Node::Leaf(leaf1));
        branch.set_branch(Nibble(5), Node::Leaf(leaf2));
        branch.set_value(b"world");

        let ext = ExtensionNode::new(&[0, 1, 0, 2, 0, 3], Node::Branch(branch));

        assert_eq!(ext.hash(), trie.hash());
    }

    
    // func TestPutExtensionMore(t *testing.T) {
    //   trie := NewTrie()
    //   trie.Put([]byte{1, 2, 3, 4}, []byte("hello1"))
    //   trie.Put([]byte{1, 2, 3, 5}, []byte("hello2"))
    //   trie.Put([]byte{1, 2, 3, 6}, []byte("world"))
    // 
    //   leaf1 := NewLeafNodeFromNibbles([]Nibble{}, []byte("hello1"))
    //   leaf2 := NewLeafNodeFromNibbles([]Nibble{}, []byte("hello2"))
    //   leaf3 := NewLeafNodeFromNibbles([]Nibble{}, []byte("world"))
    // 
    //   branch := NewBranchNode()
    //   branch.SetBranch(Nibble(4), leaf1)
    //   branch.SetBranch(Nibble(5), leaf2)
    //   branch.SetBranch(Nibble(6), leaf3)
    // 
    //   ext := NewExtensionNode([]Nibble{0, 1, 0, 2, 0, 3, 0}, branch)
    // 
    //   require.Equal(t, ext.Hash(), trie.Hash())
    // }
    #[test]
    fn test_put_extension_more() {
        let mut trie = Trie::new();
        trie.put(&[1, 2, 3, 4], b"hello1");
        trie.put(&[1, 2, 3, 5], b"hello2");
        trie.put(&[1, 2, 3, 6], b"world");

        let leaf1 = LeafNode::from_nibbles(&[], b"hello1");
        let leaf2 = LeafNode::from_nibbles(&[], b"hello2");
        let leaf3 = LeafNode::from_nibbles(&[], b"world");

        let mut branch = BranchNode::new();
        branch.set_branch(Nibble(4), Node::Leaf(leaf1));
        branch.set_branch(Nibble(5), Node::Leaf(leaf2));
        branch.set_branch(Nibble(6), Node::Leaf(leaf3));

        let ext = ExtensionNode::new(&[0, 1, 0, 2, 0, 3, 0], Node::Branch(branch));

        assert_eq!(ext.hash(), trie.hash());
    }
}
*/
