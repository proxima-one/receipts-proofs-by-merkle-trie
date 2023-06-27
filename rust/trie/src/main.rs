fn main() {

}

extern crate tiny_keccak;
use tiny_keccak::Keccak;
use std::boxed::Box;

// nodes.go

extern crate rlp;
// 
// use rlp::encode;
use std::io::Write;

// type Node interface {
//   Hash() []byte // common.Hash
//   Raw() []interface{}
// }
trait Node {
    fn hash(&self) -> Vec<u8>; 
    fn raw(&self) -> Vec<u8>;
}

// func Hash(node Node) []byte {
//   if IsEmptyNode(node) {
//     return EmptyNodeHash
//   }
//   return node.Hash()
// }
fn hash(node: &impl Node) -> Vec<u8> {
    if is_empty_node(node) {
        return EMPTY_NODE_HASH;
    }
    node.hash()
}

use std::any::Any;

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
fn serialize(node: &impl Node) -> Box<dyn Any> {
    let raw = if is_empty_node(node) {
        EMPTY_NODE_RAW
    } else {
        node.raw()
    };
    
    match encode(&raw) {
        Ok(encoded) => encoded,
        Err(err) => panic!("Error encoding: {}", err),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::vec::Vec;

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
            Nibble::from_bytes((&[1, 100]).to_vec())
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
}

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
    fn raw(&self) -> Vec<Box<dyn Node>> {
        let mut hashes: Vec<Box<dyn Node>> = Vec::with_capacity(17);
        for i in 0..16 {
            match &self.branches.borrow()[i] {
                None => hashes[i] = Box::new(EMPTY_NODE_RAW),
                Some(node) => {
                    if node.serialize().len() >= 32 {
                        hashes[i] = Box::new(node.hash());
                    } else {
                        hashes[i] = Box::new(node.raw());
                    }
                }
            }
        }

        hashes[16] = Box::new(LeafNode::new_from_bytes(&[], &self.value.borrow().as_ref().unwrap()));
        hashes
    }

    // func (b BranchNode) Serialize() []byte {
    //   return Serialize(b)
    // }
    fn serialize(&self) -> Box<dyn Any> {
        serialize(self)
    }

    // func (b BranchNode) HasValue() bool {
    //   return b.Value != nil
    // }
    fn has_value(&self) -> bool {
        self.value.borrow().is_some()
    }
}

// leaf.go

use tiny_keccak::{Keccak};

// type LeafNode struct {
//   Path  []Nibble
//   Value []byte
// }
pub struct LeafNode {
    path: Vec<Nibble>,
    value: Vec<u8>,
}

impl Node for LeafNode {
    fn hash(&self) -> Box<dyn Any> {
        self.hash()
    }

    fn raw(&self) -> Box<dyn Any> {
        self.raw()
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
            value,
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
    pub fn hash(&self) -> Box<dyn Any> {
        let mut hasher = Keccak::v256();
        let mut res = [0u8; 32];
        hasher.update(&self.serialize());
        hasher.finalize(&mut res);
        Box::new(res.to_vec())
    }

    // func (l LeafNode) Raw() []interface{} {
    //   path := ToBytes(ToPrefixed(l.Path, true))
    //   raw := []interface{}{path, l.Value}
    //   return raw
    // }
    pub fn raw(&self) -> Box<dyn Any> {
        let path_u8: Box<dyn Any> = Box::new(Nibble::to_bytes(Nibble::to_prefixed(self.path.clone(), true)));
        let value_any: Box<dyn Any> = Box::new(self.value.clone());
        let raw: Vec<Box<dyn Any>> = vec![path_u8, value_any];
        Box::new(raw)
    }

    // func (l LeafNode) Serialize() []byte {
    //   return Serialize(l)
    // }
    pub fn serialize(&self) -> Box<dyn Any> {
        serialize(self)
    }
}

// empty.go

extern crate hex;

use std::str::FromStr;

// EmptyNodeRaw     = []byte{}
static EMPTY_NODE_RAW: [u8; 0] = [];
// EmptyNodeHash, _ = hex.DecodeString("56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421")
static EMPTY_NODE_HASH: [u8; 32] = *b"56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421";

// func IsEmptyNode(node Node) bool {
//   return node == nil
// }
pub fn is_empty_node(node: &Option<Box<dyn Node>>) -> bool {
    match node {
        Some(_) => false,
        None => true,
    }
}

