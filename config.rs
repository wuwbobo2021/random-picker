use crate::*;
use std::{collections::HashMap, fmt::Display, hash::Hash, str::FromStr};

/// Alias of `HashMap`. The weight value type is always `f64`.
pub type Table<T> = HashMap<T, f64, std::hash::RandomState>;

/// Configuration required by `Picker`. All members are public
/// and are supposed to be modified by the user.
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde-config", derive(serde::Serialize, serde::Deserialize))]
pub struct Config<T: Clone + Eq + Hash> {
    /// Table of choices and weights which are proportional to the probabilities
    /// on repetitive mode or single-item mode.
    pub table: Table<T>,

    /// Do multiplicative inversion for each value in the table.
    pub inversed: bool,

    /// Allow the same item to be picked for multiple times in the result.
    pub repetitive: bool,
}

impl<T: Clone + Eq + Hash> Config<T> {
    /// Returns an invalid configuration with an empty table.
    /// Please add items into the table before using it to construct `Picker`.
    ///
    /// ```
    /// let conf = random_picker::Config::<String>::new();
    /// assert!(!conf.inversed && !conf.repetitive);
    /// assert!(conf.check().is_err());
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
            inversed: false,
            repetitive: false,
        }
    }

    /// Checks whether or not the table can be used by `Picker`.
    ///
    /// ```
    /// let mut conf: random_picker::Config<String> = "
    ///     a = -1; b = 0; c = 2
    /// ".parse().unwrap();
    /// assert!(conf.check().is_err());
    /// conf.table.insert("a".to_string(), 1.);
    /// assert!(conf.check().is_ok());
    /// conf.inversed = true;
    /// assert!(conf.check().is_err());
    /// conf.table.insert("b".to_string(), 0.1);
    /// assert!(conf.check().is_ok());
    /// ```
    pub fn check(&self) -> Result<(), Error> {
        let mut non_empty = false;
        for &v in self.table.values() {
            if v < 0. || (self.inversed && v == 0.) {
                return Err(Error::InvalidTable);
            }
            if v > 0. {
                non_empty = true;
            }
        }
        non_empty.then_some(()).ok_or(Error::InvalidTable)
    }

    /// Returns `true` if all items have equal (and valid) weight values.
    ///
    /// ```
    /// let mut conf: random_picker::Config<String> = "
    ///     a = -1; b = 1; c = 1.1
    /// ".parse().unwrap();
    /// assert!(!conf.is_fair());
    /// conf.table.insert("a".to_string(), 1.);
    /// assert!(!conf.is_fair());
    /// conf.table.insert("c".to_string(), 1.);
    /// assert!(conf.is_fair());
    /// ```
    pub fn is_fair(&self) -> bool {
        if self.check().is_err() {
            return false;
        }
        let mut v_prev = None;
        for &v in self.table.values() {
            if let Some(v_prev) = v_prev {
                if v != v_prev {
                    return false;
                }
            }
            v_prev.replace(v);
        }
        true
    }

    #[inline]
    pub(crate) fn vec_table(&self) -> Result<Vec<(T, f64)>, Error> {
        self.check()?;
        let vec = if !self.inversed {
            self.table
                .clone()
                .into_iter()
                .filter(|&(_, v)| v > 0.)
                .collect()
        } else {
            self.table
                .iter()
                .map(|(k, &v)| (k.clone(), 1. / v))
                .collect()
        };
        Ok(vec)
    }
}

impl<T: Clone + Eq + Hash> Default for Config<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl Config<String> {
    /// Appends, modifies or deletes items in the table
    /// according to the configuration input string.
    ///
    /// ```
    /// let mut conf: random_picker::Config<String> = "
    /// ## 'repetitive' and 'inversed' are special items
    /// repetitive = true
    /// inversed = false
    /// ## this line can be ignored
    /// [items]
    /// oxygen = 47
    /// silicon = 28
    /// aluminium=8; iron=5; magnesium=4;
    /// calcium=2; potassium=2; sodium=2
    /// others = 2; nonexistium = 31
    ///    aluminium 7.9; delete nonexistium
    /// ".parse().unwrap();
    /// assert_eq!(conf.table.len(), 9);
    /// assert_eq!(conf.repetitive, true);
    /// assert_eq!(conf.inversed, false);
    /// assert_eq!(conf.table.get("aluminium"), Some(&7.9));
    ///
    /// conf.append_str("\
    /// ## power_inversed/repetitive_picking without '=' are for the old format
    /// power_inversed
    /// ## invalid: repetitive = 0 (0 is not bool)
    /// repetitive = 0
    /// silicon = 28.1
    /// ");
    /// assert_eq!(conf.inversed, true);
    ///
    /// conf.append_str("inversed = false");
    /// assert_eq!(conf, random_picker::Config {
    ///     table: [
    ///         ("oxygen", 47.), ("silicon", 28.1), ("aluminium", 7.9),
    ///         ("iron", 5.), ("magnesium", 4.), ("calcium", 2.),
    ///         ("sodium", 2.), ("potassium", 2.), ("others", 2.),
    ///     ].iter().map(|&(k, v)| (k.to_string(), v)).collect(),
    ///     inversed: false,
    ///     repetitive: true
    /// });
    /// ```
    pub fn append_str(&mut self, str_items: &str) {
        for line in str_items.split(&['\r', '\n', ';']) {
            let mut spl = line.split(&[' ', '\t', '=']).filter(|s| !s.is_empty());
            let item_name;
            if let Some(s) = spl.next() {
                if let Some('#') = s.chars().nth(0) {
                    continue;
                }
                item_name = s;
            } else {
                continue;
            }

            // compatible with the old table format
            if item_name == "power_inversed" {
                self.inversed = true;
            } else if item_name == "repetitive_picking" {
                self.repetitive = true;
            } else if let Some(s) = spl.last() {
                if item_name == "delete" {
                    let _ = self.table.remove(s);
                } else if item_name == "inversed" {
                    if let Ok(b) = bool::from_str(s) {
                        self.inversed = b;
                    }
                } else if item_name == "repetitive" {
                    if let Ok(b) = bool::from_str(s) {
                        self.repetitive = b;
                    }
                } else if let Ok(v) = f64::from_str(s) {
                    self.table.insert(item_name.to_string(), v);
                }
            }
        }
    }
}

impl FromStr for Config<String> {
    type Err = Error;
    #[inline(always)]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut conf = Self::new();
        conf.append_str(s);
        Ok(conf)
    }
}

impl Display for Config<String> {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.check().is_err() {
            writeln!(f, "# INVALID!!!")?;
        }
        writeln!(f, "[random-picker]")?;
        writeln!(f, "repetitive = {}", self.repetitive)?;
        writeln!(f, "inversed = {}\n", self.inversed)?;
        writeln!(f, "[items]")?;
        format_table(f, &self.table)?;
        Ok(())
    }
}

/// Prints the weight table to the standard output.
#[inline(always)]
pub fn print_table(table: &Table<String>) {
    let mut s = String::new();
    let _ = format_table(&mut s, table);
    print!("{s}");
}

fn format_table(f: &mut impl std::fmt::Write, table: &Table<String>) -> std::fmt::Result {
    let name_len_max;
    if let Some(n) = table.keys().map(|s| s.len()).max() {
        name_len_max = n;
    } else {
        // empty?
        return Ok(());
    }

    let mut vec_table: Vec<_> = table.iter().collect();
    vec_table.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));

    for (k, v) in vec_table {
        writeln!(f, "{:>2$} = {:>9.6}", k, v, name_len_max)?;
    }
    Ok(())
}
