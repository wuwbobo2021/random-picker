use crate::*;
use rand::{rngs::OsRng, RngCore};
use std::hash::Hash;

/// Convenience wrapper for exactly one picking operation.
///
/// ```
/// let picks: Vec<String> = random_picker::pick(2,
///     "a=1;b=15;c=1.5".parse().unwrap()
/// ).unwrap();
/// // nonrepetitive by default
/// assert!(picks.iter().any(|k| k == "a" || k == "c"));
/// ```
pub fn pick<T: Clone + Eq + Hash>(amount: usize, conf: Config<T>) -> Result<Vec<T>, Error> {
    Picker::build(conf)?.pick(amount)
}

/// Generator of groups of random items of type `T` with different probabilities.
/// According to the configuration, items in each group can be either
/// repetitive or non-repetitive.
pub struct Picker<T: Clone + Eq + Hash, R: RngCore> {
    rng: R,
    table: Vec<(T, f64)>,
    grid: Vec<f64>,
    grid_width: f64,
    repetitive: bool,

    // read it after calling `pick_indexes()`
    picked_indexes: Vec<usize>,
}

impl<T: Clone + Eq + Hash> Picker<T, OsRng> {
    /// Builds the `Picker` with given configuration, using the OS random source.
    pub fn build(conf: Config<T>) -> Result<Self, Error> {
        Picker::build_with_rng(conf, OsRng)
    }
}

impl<T: Clone + Eq + Hash, R: RngCore> Picker<T, R> {
    /// Builds the `Picker` with given configuration and the given random source.
    pub fn build_with_rng(conf: Config<T>, rng: R) -> Result<Self, Error> {
        let table = conf.vec_table()?;
        let table_len = table.len();

        let mut grid = Vec::with_capacity(table_len);
        let mut cur = 0.;
        for (_, val) in &table {
            cur += val;
            grid.push(cur);
        }
        let grid_width = *grid.last().unwrap();

        Ok(Self {
            rng,
            table,
            grid,
            grid_width,
            repetitive: conf.repetitive,
            picked_indexes: Vec::with_capacity(table_len),
        })
    }

    /// Returns the size of the weight table that contains all possible choices (p > 0).
    ///
    /// ```
    /// use random_picker::Picker;
    /// let mut conf: random_picker::Config<String> = "
    ///     a = 0; b = 1; c = 1.1
    /// ".parse().unwrap();
    /// let picker = Picker::build(conf.clone()).unwrap();
    /// assert_eq!(picker.table_len(), 2);
    /// conf.append_str("b = 0; c = 0");
    /// assert!(Picker::build(conf).is_err());
    /// ```
    #[inline(always)]
    pub fn table_len(&self) -> usize {
        self.table.len()
    }

    /// Picks `amount` of items and returns the group of items.
    /// `amount` must not exceed `table_len()`.
    #[inline(always)]
    pub fn pick(&mut self, amount: usize) -> Result<Vec<T>, Error> {
        self.pick_indexes(amount)?;
        Ok(self
            .picked_indexes
            .iter()
            .map(|&i| self.item_key(i))
            .collect())
    }

    /// Picks `dest.len()` of items and writes them into `dest` (avoids allocation).
    /// Length of `dest` must not exceed `table_len()`.
    #[inline]
    pub fn write_to(&mut self, dest: &mut [T]) -> Result<(), Error> {
        self.pick_indexes(dest.len())?;
        for (i, k) in dest.iter_mut().enumerate() {
            *k = self.item_key(self.picked_indexes[i]);
        }
        Ok(())
    }

    /// Evaluates probabilities of existences of table items in each group
    /// of length `amount`, by generating groups of items for `test_times`.
    ///
    /// ```
    /// use random_picker::*;
    /// let mut conf: Config<String> = "
    ///     a=856; b=139; c=297; d=378; e=1304;
    ///     f=289; g=199; h=528; i=627; j=  13;
    ///     k= 42; l=339; m=249; n=707; o= 797;
    ///     p=199; q= 12; r=677; s=607; t=1045;
    ///     u=249; v= 92; w=149; x= 17; y= 199; z=8;
    /// ".parse().unwrap();
    /// assert_eq!(conf.repetitive, false);
    /// assert_eq!(conf.table.len(), 26);
    /// let table_probs = conf.calc_probabilities(3).unwrap();
    ///
    /// let mut picker = Picker::build(conf.clone()).unwrap();
    /// let table_freqs = picker.test_freqs(3, 1_000_000).unwrap();
    /// for (k, v) in table_freqs.iter() {
    ///     assert!((*v - *table_probs.get(k).unwrap()).abs() < 0.005);
    /// }
    ///
    /// conf.append_str("repetitive = true");
    /// assert_eq!(conf.repetitive, true);
    /// let table_probs = conf.calc_probabilities(3).unwrap();;
    ///
    /// let mut picker = Picker::build_with_rng(conf, rand::thread_rng()).unwrap();
    /// let table_freqs = picker.test_freqs(3, 1_000_000).unwrap();
    /// for (k, v) in table_freqs.iter() {
    ///     assert!((*v - *table_probs.get(k).unwrap()).abs() < 0.005);
    /// }
    /// ```
    pub fn test_freqs(&mut self, amount: usize, test_times: usize) -> Result<Table<T>, Error> {
        let mut tbl_freq = vec![0_usize; self.table_len()];
        if !self.repetitive {
            for _ in 0..test_times {
                self.pick_indexes(amount)?;
                for &idx in &self.picked_indexes {
                    tbl_freq[idx] += 1;
                }
            }
        } else {
            let mut tbl_picked = vec![false; self.table_len()];
            for _ in 0..test_times {
                for b in tbl_picked.iter_mut() {
                    *b = false;
                }
                self.pick_indexes(amount)?;
                for &idx in &self.picked_indexes {
                    if !tbl_picked[idx] {
                        tbl_freq[idx] += 1;
                        tbl_picked[idx] = true;
                    }
                }
            }
        }

        let test_times = test_times as f64;
        let table = tbl_freq
            .iter()
            .enumerate()
            .map(|(i, &v)| (self.item_key(i), v as f64 / test_times))
            .collect();
        Ok(table)
    }

    /// Picks `amount` of indexes and replaces values in `self.picked_indexes`.
    #[inline]
    fn pick_indexes(&mut self, amount: usize) -> Result<(), Error> {
        if !self.repetitive && amount > self.table_len() {
            return Err(Error::InvalidAmount);
        }
        self.picked_indexes.clear();

        let mut tbl_picked = vec![false; self.table_len()];
        while self.picked_indexes.len() < amount {
            let i = self.pick_index()?;
            if !self.repetitive {
                if tbl_picked[i] {
                    continue;
                }
                tbl_picked[i] = true;
            }
            self.picked_indexes.push(i);
        }
        Ok(())
    }

    #[inline(always)]
    fn pick_index(&mut self) -> Result<usize, Error> {
        let mut bytes = [0u8; 4];
        self.rng
            .try_fill_bytes(&mut bytes)
            .map_err(Error::RandError)?;

        let val = (u32::from_ne_bytes(bytes) as f64) / (u32::MAX as f64) * self.grid_width;
        for (i, &v) in self.grid.iter().enumerate() {
            if val <= v {
                return Ok(i);
            };
        }

        Ok(self.table_len() - 1) // almost impossible
    }

    #[inline(always)]
    fn item_key(&self, i: usize) -> T {
        self.table[i].0.clone()
    }
}
