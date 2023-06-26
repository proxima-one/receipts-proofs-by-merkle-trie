mod trie;

extern crate tiny_keccak;
use tiny_keccak::Keccak;

fn main() {

}

// trie.go

#[derive(Clone)]
pub enum Node {
    Empty,
    Leaf(LeafNode),
    Branch(BranchNode),
    Extension(ExtensionNode),
}

pub struct Trie {
    root: Node,
}

impl Trie {
    pub fn new() -> Trie {
        Trie { root: Node::Empty }
    }

    pub fn hash(&self) -> Vec<u8> {
        match &self.root {
            Node::Empty => empty_node_hash(),
            _ => self.root.hash(),
        }
    }

    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        let mut node = &self.root;
        let mut nibbles = from_bytes(key);

        loop {
            match node {
                Node::Empty => return None,
                Node::Leaf(leaf) => {
                    let matched = prefix_matched_len(&leaf.path, &nibbles);
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
                    let matched = prefix_matched_len(&ext.path, &nibbles);
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
        let mut nibbles = from_bytes(key);

        loop {
            if is_empty_node(&node) {
                *node = Box::new(LeafNode::new_leaf_node_from_nibbles(nibbles.clone(), value.to_vec()));
                return Ok(());
            }

            if let Node::Leaf(ref mut leaf) = node {
                let matched = prefix_matched_len(&leaf.path, &nibbles);

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
                    *node = Box::new(new_extension_node(leaf.path[..matched].to_vec(), Box::new(branch)));
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
                let matched = prefix_matched_len(&ext.path, &nibbles);
                if matched < ext.path.len() {
                    let (ext_nibbles, remaining) = ext.path.split_at(matched);
                    let (branch_nibble, ext_remaining_nibbles) = remaining.split_at(1);
                    let mut branch = BranchNode::new();

                    if ext_remaining_nibbles.is_empty() {
                        branch.set_branch(branch_nibble[0], ext.next.clone());
                    } else {
                        branch.set_branch(branch_nibble[0], Box::new(new_extension_node(ext_remaining_nibbles.to_vec(), ext.next.clone())));
                    }

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
                        *node = Box::new(new_extension_node(ext_nibbles.to_vec(), Box::new(branch)));
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

pub struct Nibble(u8);

impl Nibble {
    fn is_nibble(nibble: u8) -> bool {
        let n = nibble as usize;
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

pub struct BranchNode {
    // Branches [16]Node
    branches: RefCell<[Option<Box<dyn Node>>; 16]>,
    // Value    []byte
    value: RefCell<Option<Vec<u8>>>,
}

impl BranchNode {
    // func NewBranchNode() *BranchNode
    fn new() -> BranchNode {
        BranchNode {
            branches: RefCell::new([None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None]),
            value: RefCell::new(None),
        }
    }

    // func (b BranchNode) Hash() []byte
    fn hash(&self) -> Vec<u8> {
        let data = self.serialize();
        let mut keccak = Keccak::new_keccak256();
        let mut hash = [0u8; 32];
        keccak.update(&data);
        keccak.finalize(&mut hash);
        hash.to_vec()
    }

    // func (b *BranchNode) SetBranch(nibble Nibble, node Node)
    fn set_branch(&self, nibble: Nibble, node: Box<dyn Node>) {
        self.branches.borrow_mut()[nibble as usize] = Some(node);
    }

    // func (b *BranchNode) RemoveBranch(nibble Nibble)
    fn remove_branch(&self, nibble: Nibble) {
        self.branches.borrow_mut()[nibble as usize] = None;
    }

    // func (b *BranchNode) SetValue(value []byte)
    fn set_value(&self, value: Vec<u8>) {
        *self.value.borrow_mut() = Some(value);
    }

    // func (b *BranchNode) RemoveValue()
    fn remove_value(&self) {
        *self.value.borrow_mut() = None;
    }

    // func (b BranchNode) Raw() []interface{}
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

    // func (b BranchNode) Serialize() []byte
    fn serialize(&self) -> Vec<u8> {
        serialize(self)
    }

    // func (b BranchNode) HasValue() bool
    fn has_value(&self) -> bool {
        self.value.borrow().is_some()
    }
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

// leaf.go

use tiny_keccak::{Hasher, Keccak};

#[derive(Debug)]
pub struct LeafNode {
    path: Vec<Nibble>,
    value: Vec<u8>,
}

impl LeafNode {
    // func NewLeafNodeFromNibbleBytes(nibbles []byte, value []byte) (*LeafNode, error)
    pub fn new_from_nibble_bytes(nibbles: &[u8], value: &[u8]) -> Result<LeafNode, &'static str> {
        let ns = from_nibble_bytes(nibbles)?;
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
        LeafNode::new_leaf_node_from_nibbles(from_bytes(key), value.to_vec())
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
        (to_bytes(&to_prefixed(&self.path, true)), self.value.clone())
    }

    // func (l LeafNode) Serialize() []byte
    pub fn serialize(&self) -> Vec<u8> {
        serialize(self.raw())
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

// nodes.go

extern crate rlp;

use rlp::encode;
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
