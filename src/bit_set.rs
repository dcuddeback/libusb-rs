type Item = u32;
const ITEM_BITS: usize = Item::BITS as usize;

#[derive(Clone)]
pub struct BitSet {
    nbits: usize,
    storage: Vec<Item>,
}

impl<'a> BitSet {
    #[inline]
    pub fn with_capacity(nbits: usize) -> BitSet {
        return BitSet {
            nbits,
            storage: vec![0 as Item; (nbits + ITEM_BITS - 1) / ITEM_BITS],
        };
    }

    #[inline]
    pub fn contains(&self, value: usize) -> ::Result<bool> {
        if value < self.nbits {
            Ok(self.storage[value / ITEM_BITS] & (1 as Item) << (value % ITEM_BITS) != 0)
        } else {
            Err(::Error::Overflow)
        }
    }

    #[inline]
    pub fn insert(&mut self, value: usize) -> ::Result<()> {
        if !self.contains(value)? {
            self.storage[value / ITEM_BITS] |= (1 as Item) << (value % ITEM_BITS);
        }
        Ok(())
    }

    #[inline]
    pub fn remove(&mut self, value: usize) -> ::Result<()> {
        if self.contains(value)? {
            self.storage[value / ITEM_BITS] &= !((1 as Item) << (value % ITEM_BITS));
        }
        Ok(())
    }

    #[inline]
    pub fn iter(&'a self) -> Iter<'a> {
        return Iter {
            data: &self.storage,
            offset: 0,
            block: self.storage[0],
        };
    }
}

#[derive(Clone)]
pub struct Iter<'a> {
    data: &'a Vec<Item>,
    offset: usize,
    block: Item,
}

impl<'a> Iterator for Iter<'a> {
    type Item = usize;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            while self.block != 0 {
                let bit = self.block.trailing_zeros() as Self::Item;
                self.block &= self.block - 1;
                return Some(self.offset + bit);
            }

            self.offset += ITEM_BITS;

            if self.offset < self.data.len() * ITEM_BITS {
                self.block = self.data[self.offset / ITEM_BITS];
            } else {
                break;
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_checks_capacity() {
        let mut set = BitSet::with_capacity(256);

        assert!(matches!(set.contains(123), Ok(false)));
        assert!(matches!(set.contains(456), Err(::Error::Overflow)));

        assert!(matches!(set.insert(123), Ok(_)));
        assert!(matches!(set.insert(456), Err(::Error::Overflow)));

        assert!(matches!(set.remove(123), Ok(_)));
        assert!(matches!(set.remove(456), Err(::Error::Overflow)));
    }

    #[test]
    fn it_inserts() {
        let mut set = BitSet::with_capacity(256);

        assert!(matches!(set.contains(123), Ok(false)));
        assert!(matches!(set.insert(123), Ok(())));
        assert!(matches!(set.contains(123), Ok(true)));
    }

    #[test]
    fn it_inserts_existing() {
        let mut set = BitSet::with_capacity(256);
        set.insert(123).unwrap();

        assert!(matches!(set.contains(123), Ok(true)));
        assert!(matches!(set.insert(123), Ok(())));
        assert!(matches!(set.contains(123), Ok(true)));
    }

    #[test]
    fn it_removes() {
        let mut set = BitSet::with_capacity(256);
        set.insert(123).unwrap();

        assert!(matches!(set.contains(123), Ok(true)));
        assert!(matches!(set.remove(123), Ok(())));
        assert!(matches!(set.contains(123), Ok(false)));
    }

    #[test]
    fn it_removes_nonexistent() {
        let mut set = BitSet::with_capacity(256);

        assert!(matches!(set.contains(123), Ok(false)));
        assert!(matches!(set.remove(123), Ok(())));
        assert!(matches!(set.contains(123), Ok(false)));
    }

    #[test]
    fn it_iterates() {
        let mut set = BitSet::with_capacity(256);

        let data: Vec<usize> = set.iter().collect();
        assert_eq!(data, []);

        set.insert(1).unwrap();
        set.insert(12).unwrap();
        set.insert(123).unwrap();
        set.insert(255).unwrap();
        set.insert(512).unwrap_err();

        let data: Vec<usize> = set.iter().collect();
        assert_eq!(data, [1, 12, 123, 255]);
    }
}
