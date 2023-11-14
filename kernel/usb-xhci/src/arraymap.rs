pub struct ArrayMap<K, V, const N: usize>
where
    K: Copy + Eq,
    V: Copy + Eq,
{
    arr: [Option<(K, V)>; N]
}

impl<K, V, const N: usize> ArrayMap<K, V, N>
where
    K: Copy + Eq,
    V: Copy + Eq,
{
    pub fn new() -> Self {
        Self {
            arr: core::array::from_fn(|_| None)
        }
    }

    pub fn get(&self, k: K) -> Option<V> {
        self.arr.iter().find_map(|ent| {
            ent.map(|(key, value)| {
                if k == key { Some(value) } else { None }
            }).flatten()
        })
    }

    pub fn set(&mut self, k: K, v: V) -> bool {
        self.arr.iter_mut().find_map(|ent| {
            if ent.is_none() {
                *ent = Some((k, v));
                Some(())
            } else {
                None
            }
        }).is_some()
    }

    pub fn delete(&mut self, k: K) -> bool {
        self.arr.iter_mut().find_map(|ent| {
            if let Some((key, _)) = *ent {
                if k == key {
                    *ent = None;
                    return Some(());
                }
            }
            None
        }).is_some()
    }

    /// Get and delete at once.
    pub fn take(&mut self, k: K) -> Option<V> {
        self.arr.iter_mut().find_map(|ent| {
            if let Some((key, value)) = *ent {
                if k == key {
                    *ent = None;
                    return Some(value);
                }
            }
            None
        })
    }
}