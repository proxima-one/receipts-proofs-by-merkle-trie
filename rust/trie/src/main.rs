fn main() {

}

extern crate tiny_keccak;
use tiny_keccak::Keccak;


// nodes.go

// extern crate rlp;
// 
// use rlp::encode;
use std::io::Write;

trait Node {
    fn hash(&self) -> Vec<u8>; 
    fn raw(&self) -> Vec<u8>;
}

// func Hash(node Node) []byte
fn hash(node: &impl Node) -> Vec<u8> {
    if is_empty_node(node) {
        return EMPTY_NODE_HASH;
    }
    node.hash()
}

// func Serialize(node Node) []byte
fn serialize(node: &impl Node) -> Vec<u8> {
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
    pub fn new() -> Trie {
        Trie { root: Node::Empty }
    }

    pub fn hash(&self) -> Vec<u8> {
        match &self.root {
            Node::Empty => EMPTY_NODE_HASH,
            _ => self.root.hash(),
        }
    }

    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        let mut node = &self.root;
        let mut nibbles = Nibble::from_bytes(key);

        loop {
            match node {
                <dyn Node>::Empty => return None,
                Node::Leaf(leaf) => {
                    let matched = Nibble::prefix_matched_len(&leaf.path, &nibbles);
                    if matched != leaf.path.len() || matched != nibbles.len() {
                        return None;
                    }
                    return Some(leaf.value.clone());
                }
                Node::Branch(branch) => {
                    if nibbles.is_empty() {
                        return branch.value.clone();
                    }
                    let (b, remaining) = nibbles.split_at(1);
                    nibbles = remaining.to_vec();
                    node = &branch.branches[b[0] as usize];
                }
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
        let mut node = &mut self.root;
        let mut nibbles = Nibble::from_bytes(key);

        loop {
            if is_empty_node(&node) {
                *node = Box::new(LeafNode::new_leaf_node_from_nibbles(nibbles.clone(), value.to_vec()));
                return Ok(());
            }

            if let Node::Leaf(ref mut leaf) = node {
                let matched = Nibble::prefix_matched_len(&leaf.path, &nibbles);

                if matched == nibbles.len() && matched == leaf.path.len() {
                    *node = Box::new(LeafNode::new_leaf_node_from_nibbles(leaf.path.clone(), value.to_vec()));
                    return Ok(());
                }

                let mut branch = BranchNode::new();

                if matched == leaf.path.len() {
                    branch.set_value(leaf.value.clone());
                }

                if matched == nibbles.len() {
                    branch.set_value(value.to_vec());
                }

                if matched > 0 {
                    *node = Box::new(ExtensionNode::new(leaf.path[..matched].to_vec(), Box::new(branch)));
                } else {
                    *node = Box::new(branch);
                }

                if matched < leaf.path.len() {
                    let (branch_nibble, leaf_nibbles) = leaf.path.split_at(matched + 1);
                    branch.set_branch(branch_nibble[0], Box::new(LeafNode::new_leaf_node_from_nibbles(leaf_nibbles.to_vec(), leaf.value.clone())));
                }

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

            if let Node::Extension(ref mut ext) = node {
                let matched = Nibble::prefix_matched_len(&ext.path, &nibbles);
                if matched < ext.path.len() {
                    let (ext_nibbles, remaining) = ext.path.split_at(matched);
                    let (branch_nibble, ext_remaining_nibbles) = remaining.split_at(1);
                    let mut branch = BranchNode::new();

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

                    if ext_nibbles.is_empty() {
                        *node = Box::new(branch);
                    } else {
                        *node = Box::new(ExtensionNode::new(ext_nibbles.to_vec(), Box::new(branch)));
                    }
                    return Ok(());
                }

                let (_, remaining) = nibbles.split_at(matched);
                nibbles = remaining.to_vec();
                node = &mut ext.next;
                continue;
            }
            
            return Err("Unknown type");
        }
    }
}

// nibbles.go
#[derive(Clone)]
pub struct Nibble(u8);

impl Nibble {
    fn to_usize(&self) -> usize {
        self.0 as usize
    }

    fn is_nibble(nibble: u8) -> bool {
        let n = nibble.to_usize();
        n >= 0 && n < 16
    }

    fn from_nibble_byte(n: u8) -> Result<Nibble, &'static str> {
        if !Nibble::is_nibble(n) {
            return Err("Non-nibble byte");
        }
        Ok(Nibble(n))
    }

    fn from_nibble_bytes(nibbles: Vec<u8>) -> Result<Vec<Nibble>, &'static str> {
        let mut ns = Vec::with_capacity(nibbles.len());
        for n in nibbles {
            let nibble = Nibble::from_nibble_byte(n)?;
            ns.push(nibble);
        }
        Ok(ns)
    }

    fn from_byte(b: u8) -> Vec<Nibble> {
        vec![
            Nibble((b >> 4) as u8),
            Nibble((b % 16) as u8),
        ]
    }

    fn from_bytes(bs: Vec<u8>) -> Vec<Nibble> {
        let mut ns = Vec::with_capacity(bs.len() * 2);
        for b in bs {
            ns.extend(Nibble::from_byte(b));
        }
        ns
    }

    fn from_string(s: String) -> Vec<Nibble> {
        Nibble::from_bytes(s.into_bytes())
    }

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

    // ToBytes converts a slice of nibbles to a byte slice
    // assuming the nibble slice has even number of nibbles.
    // func ToBytes(ns []Nibble) []byte
    fn to_bytes(ns: Vec<Nibble>) -> Vec<u8> {
        let mut buf = Vec::with_capacity(ns.len() / 2);

        for ns_chunk in ns.chunks_exact(2) {
            let b = (ns_chunk[0].0 << 4) + ns_chunk[1].0;
            buf.push(b);
        }

        buf
    }

    // [0,1,2,3], [0,1,2] => 3
    // [0,1,2,3], [0,1,2,3] => 4
    // [0,1,2,3], [0,1,2,3,4] => 4
    // func PrefixMatchedLen(node1 []Nibble, node2 []Nibble) int
    fn prefix_matched_len(node1: &[Nibble], node2: &[Nibble]) -> usize {
        node1.iter().zip(node2.iter()).take_while(|&(n1, n2)| n1 == n2).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_nibble() {
        for i in 0..20 {
            let is_nibble = i >= 0 && i < 16;
            assert_eq!(is_nibble, is_nibble(i as u8), "{}", i);
        }
    }

    #[test]
    fn test_to_prefixed() {
        let cases = [
            (vec![1], false, vec![1, 1]),
            (vec![1, 2], false, vec![0, 0, 1, 2]),
            (vec![1], true, vec![3, 1]),
            (vec![1, 2], true, vec![2, 0, 1, 2]),
            (vec![5, 0, 6], true, vec![3, 5, 0, 6]),
            (vec![14, 3], false, vec![0, 0, 14, 3]),
            (vec![9, 3, 6, 5], true, vec![2, 0, 9, 3, 6, 5]),
            (vec![1, 3, 3, 5], true, vec![2, 0, 1, 3, 3, 5]),
            (vec![7], true, vec![3, 7]),
        ];

        for (ns, is_leaf_node, expected) in cases.iter() {
            assert_eq!(&expected[..], &Nibble::to_prefixed(ns.as_slice(), *is_leaf_node)[..]);
        }
    }

    #[test]
    fn test_from_bytes() {
        // [1, 100] -> ['0x01', '0x64']
        assert_eq!(vec![0, 1, 6, 4], Nibble::from_bytes(&[1, 100]));
    }

    #[test]
    fn test_to_bytes() {
        let bytes = &[0, 1, 2, 3];
        assert_eq!(bytes, &Nibble::to_bytes(&Nibble::from_bytes(bytes))[..]);
    }

    #[test]
    fn test_prefix_matched_len() {
        assert_eq!(3, Nibble::prefix_matched_len(&[0, 1, 2, 3], &[0, 1, 2]));
        assert_eq!(4, Nibble::prefix_matched_len(&[0, 1, 2, 3], &[0, 1, 2, 3]));
        assert_eq!(4, Nibble::prefix_matched_len(&[0, 1, 2, 3], &[0, 1, 2, 3, 4]));
    }
}

// extension.go

pub struct ExtensionNode {
    path: Vec<Nibble>,
    next: Box<dyn Node>,
}

impl ExtensionNode {
    // func NewExtensionNode(nibbles []Nibble, next Node) *ExtensionNode
    fn new(nibbles: Vec<Nibble>, next: Box<dyn Node>) -> ExtensionNode {
        ExtensionNode {
            path: nibbles,
            next: next,
        }
    }

    // func (e ExtensionNode) Hash() []byte
    fn hash(&self) -> Vec<u8> {
        let data = self.serialize();
        let mut keccak = Keccak::new_keccak256();
        let mut hash = [0u8; 32];
        keccak.update(&data);
        keccak.finalize(&mut hash);
        hash.to_vec()
    }

    // func (e ExtensionNode) Raw() []interface{}
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
    
    // func (e ExtensionNode) Serialize() []byte
    fn serialize(&self) -> Vec<u8> {
        serialize(self)
    }
}

// branch.go

use std::cell::RefCell;

// type BranchNode struct {
// 	Branches [16]Node
// 	Value    []byte
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

    fn serialize(&self) -> Vec<u8> {
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
                None => hashes.push(Box::new(EmptyNodeRaw)), // Ваш код показывает, что вы используете некий `EmptyNodeRaw`. Замените это на соответствующий тип в вашем коде.
                Some(node) => {
                    if node.serialize().len() >= 32 {
                        hashes.push(Box::new(node.hash()));
                    } else {
                        hashes.push(Box::new(node.raw()));
                    }
                }
            }
        }

        match &*self.value.borrow() {
            None => hashes.push(Box::new(Vec::<u8>::new())),
            Some(value) => hashes.push(Box::new(value.clone())),
        }

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

// leaf.go

use tiny_keccak::{Keccak};

#[derive(Debug)]
pub struct LeafNode {
    path: Vec<Nibble>,
    value: Vec<u8>,
}

impl Node for LeafNode {
    fn hash(&self) -> Vec<u8> {
        self.hash()
    }

    fn raw(&self) -> Vec<u8> {
        let (path, value) = self.raw();
        serialize((path, value))
    }
}

impl LeafNode {
    // func NewLeafNodeFromNibbleBytes(nibbles []byte, value []byte) (*LeafNode, error)
    pub fn new_from_nibble_bytes(nibbles: &[u8], value: &[u8]) -> Result<LeafNode, &'static str> {
        let ns = Nibble::from_nibble_bytes(nibbles)?;
        Ok(LeafNode {
            path: ns,
            value: value.to_vec(),
        })
    }

    // func NewLeafNodeFromNibbles(nibbles []Nibble, value []byte) *LeafNode
    pub fn new_leaf_node_from_nibbles(nibbles: Vec<Nibble>, value: Vec<u8>) -> LeafNode {
        LeafNode {
            path: nibbles,
            value,
        }
    }

    // func NewLeafNodeFromKeyValue(key, value string) *LeafNode
    pub fn new_from_key_value(key: &str, value: &str) -> LeafNode {
        LeafNode::new_from_bytes(key.as_bytes(), value.as_bytes())
    }

    // func NewLeafNodeFromBytes(key, value []byte) *LeafNode
    pub fn new_from_bytes(key: &[u8], value: &[u8]) -> LeafNode {
        LeafNode::new_leaf_node_from_nibbles(Nibble::from_bytes(key), value.to_vec())
    }

    // func (l LeafNode) Hash() []byte
    pub fn hash(&self) -> Vec<u8> {
        let mut hasher = Keccak::v256();
        let mut res = [0u8; 32];
        hasher.update(&self.serialize());
        hasher.finalize(&mut res);
        res.to_vec()
    }

    // func (l LeafNode) Raw() []interface{}
    pub fn raw(&self) -> (Vec<u8>, Vec<u8>) {
        (Nibble::to_bytes(Nibble::to_prefixed(&self.path, true)), self.value.clone())
    }

    // func (l LeafNode) Serialize() []byte
    pub fn serialize(&self) -> Vec<u8> {
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

// func IsEmptyNode(node Node) bool
pub fn is_empty_node(node: &Option<Box<dyn Node>>) -> bool {
    match node {
        Some(_) => false,
        None => true,
    }
}

