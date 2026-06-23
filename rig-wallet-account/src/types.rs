#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Curve {
    K256,
    Ed25519,
}

impl Curve {
    pub fn seed_key(&self) -> &[u8] {
        match self {
            Curve::K256 => b"Bitcoin seed",
            Curve::Ed25519 => b"ed25519 seed",
        }
    }
}
